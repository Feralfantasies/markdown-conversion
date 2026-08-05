//! Integration tests for the story API.
//!
//! These tests start a real Axum server (in-process, bound to an ephemeral
//! port) that serves the `test_story` fixture, then exercise it over HTTP.
//!
//! Run with:
//!   cargo test                      (assertions only)
//!   cargo test -- --nocapture       (with the exploration summary output)

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use markdown_converter::models::Page;
use markdown_converter::{create_router, AppState};

/// Start the story API server on an ephemeral port and return its base URL.
///
/// Binding to port 0 lets the OS assign a free port, so multiple tests can run
/// in parallel without conflicts.
async fn start_server() -> String {
    let story_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_story");
    let state = AppState::new(&story_dir, "starting_point");
    let router = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind to ephemeral port");
    let addr = listener.local_addr().expect("no local address");

    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("server failed");
    });

    format!("http://{}", addr)
}

// ── Test 1: Base URL response validation ────────────────────────────────────

#[tokio::test]
async fn base_url_returns_expected_entry_point() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/", base))
        .send()
        .await
        .expect("failed to reach base URL");

    assert_eq!(resp.status(), 200, "GET / should return HTTP 200");

    let page: Page = resp
        .json()
        .await
        .expect("response should be valid Page JSON");

    // Value checks against the test_story fixture.
    assert_eq!(page.filename, "starting_point", "entry point filename");
    assert_eq!(page.title, "Starting Point", "entry point title");
    assert!(!page.story.is_empty(), "entry point should have story text");

    // The entry page must expose exactly the road, forest, and self choices.
    let links: HashSet<String> = page.choices.iter().map(|c| c.link.clone()).collect();
    assert_eq!(
        links,
        HashSet::from(["road".to_string(), "forest".to_string(), "self".to_string()]),
        "entry point should link to road, forest, and self, got {:?}",
        links
    );

    // The self-link choice must carry its action payload through to the client.
    let watch = page
        .choices
        .iter()
        .find(|c| c.link == "self")
        .expect("self choice should exist");
    assert_eq!(watch.cost, Some(0.0), "self choice cost");
    assert_eq!(
        watch.functions,
        Some(vec!["current_time=get_time()".to_string()]),
        "self choice should carry the get_time function"
    );
    assert!(
        watch.msg.as_deref().unwrap_or("").contains("{}"),
        "self choice msg should contain a placeholder"
    );
    assert_eq!(
        watch.msg_var.as_deref(),
        Some("current_time"),
        "self choice msg should reference the current_time variable"
    );

    // The navigation choices must carry their function arrays through.
    let road = page
        .choices
        .iter()
        .find(|c| c.link == "road")
        .expect("road choice should exist");
    assert_eq!(
        road.functions,
        Some(vec![
            "set_character_class(road_warrior)".to_string(),
            "set_background(road.png)".to_string(),
        ]),
        "road choice should carry set_character_class + set_background"
    );

    println!("base_url OK: {} choices -> {:?}", page.choices.len(), links);
}

// ── Test 2: Explore every unique option (loop-safe BFS) ─────────────────────

#[tokio::test]
async fn explores_every_unique_option_exactly_once() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    // (page, link) pairs we have already followed.
    let mut followed_options: HashSet<(String, String)> = HashSet::new();
    // How many times each (page, link) pair is *encountered* (loops > 1).
    let mut occurrence_count: HashMap<(String, String), usize> = HashMap::new();
    // Pages we have successfully fetched.
    let mut visited_pages: HashSet<String> = HashSet::new();
    // Ordered log of every step taken.
    let mut traversal_log: Vec<(String, String, String)> = Vec::new(); // (from, display, to)

    // BFS queue holds (page_id, path_so_far).
    let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();

    // Seed with the entry point.
    let entry: Page = client
        .get(format!("{}/", base))
        .send()
        .await
        .expect("failed to reach base URL")
        .json()
        .await
        .expect("entry should be valid JSON");

    let entry_id = entry.filename.clone();
    visited_pages.insert(entry_id.clone());
    queue.push_back((entry_id.clone(), vec![entry_id.clone()]));

    let mut total_encountered: usize = 0;
    let mut total_followed: usize = 0;

    while let Some((page_id, path)) = queue.pop_front() {
        // Mark this page as visited (it was enqueued via a followed option).
        visited_pages.insert(page_id.clone());

        // Fetch the page (we re-fetch for simplicity; the server is stateless).
        let url = if page_id == entry_id {
            format!("{}/", base)
        } else {
            format!("{}/pages/{}", base, page_id)
        };

        let page: Page = client
            .get(&url)
            .send()
            .await
            .unwrap_or_else(|e| panic!("failed to fetch {}: {}", url, e))
            .json()
            .await
            .unwrap_or_else(|e| panic!("invalid JSON from {}: {}", url, e));

        for choice in &page.choices {
            let link = choice.link.clone();
            let option_key = (page_id.clone(), link.clone());
            total_encountered += 1;
            *occurrence_count.entry(option_key.clone()).or_insert(0) += 1;

            if followed_options.contains(&option_key) {
                // Already took this option; it's a loop — count it, skip it.
                continue;
            }

            // Mark this option as followed so we never pick it again.
            followed_options.insert(option_key);
            total_followed += 1;
            traversal_log.push((page_id.clone(), choice.display.clone(), link.clone()));

            // "self" links point back to the same page; no new page to enqueue.
            if link == "self" {
                continue;
            }

            let mut step_path = path.clone();
            step_path.push(link.clone());
            queue.push_back((link, step_path));
        }
    }

    // ── Reachability check ───────────────────────────────────────────────
    let all_pages: Vec<String> = client
        .get(format!("{}/pages", base))
        .send()
        .await
        .expect("failed to list pages")
        .json()
        .await
        .expect("/pages should return a JSON array");

    let unreachable: Vec<&String> = all_pages
        .iter()
        .filter(|p| !visited_pages.contains(*p))
        .collect();
    assert!(
        unreachable.is_empty(),
        "unreachable pages: {:?}",
        unreachable
    );

    // ── Loop detection assertions ────────────────────────────────────────
    // Each of the 7 unique options is encountered at least once, and the
    // repeated links back to already-visited pages are encountered again.
    let repeated: Vec<_> = occurrence_count
        .iter()
        .filter(|(_, &count)| count > 1)
        .collect();
    assert!(
        !repeated.is_empty(),
        "the fixture contains loops, so some option must be encountered twice"
    );
    assert_eq!(
        occurrence_count.len(),
        7,
        "should encounter 7 unique (page, link) pairs"
    );

    // The test_story fixture is fully connected (a triangle) plus a self-loop
    // on the entry page, so we expect exactly 7 unique directed options
    // (6 page links + 1 self) and 3 visited pages.
    assert_eq!(visited_pages.len(), 3, "should visit all 3 pages");
    assert_eq!(total_followed, 7, "should follow 7 unique options");
    assert!(
        total_encountered > total_followed,
        "loops should be detected (encountered {} > followed {})",
        total_encountered,
        total_followed
    );

    // ── Summary (visible with `cargo test -- --nocapture`) ─────────────
    println!("exploration OK: {} pages visited", visited_pages.len());
    println!("  options encountered    : {}", total_encountered);
    println!("  unique options followed: {}", total_followed);
    println!(
        "  loops detected/skipped : {}",
        total_encountered - total_followed
    );
    println!("  traversal order:");
    for (i, (from, display, to)) in traversal_log.iter().enumerate() {
        println!("    {}. [{}] \"{}\" -> [{}]", i + 1, from, display, to);
    }
    println!("  option occurrence counts:");
    let mut counts: Vec<_> = occurrence_count.iter().collect();
    counts.sort();
    for ((page, link), count) in counts {
        let flag = if *count > 1 { "LOOP" } else { "ok" };
        println!("    [{}] {} -> {} ({}x)", flag, page, link, count);
    }
}
