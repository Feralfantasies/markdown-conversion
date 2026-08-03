# Starting Point

This is where the adventure begins, enter in any text you want and it will be rendered out to the screen, then change the options below to change where the buttons go

### Choices

- [Explore the road]([[road.md]]) {
  cost=10,
  functions=[
  set_character_class(road_warrior),
  set_background(road.png)
  ]
  }
- [Explore the forest]([[forest.md]]) {
  cost=10,
  functions=[
  set_character_class(forest_wizard),
  set_background(forest.png)
  ]
  }
- [Check your watch](self) {
  cost=0,
  functions=[
  current_time=get_time()
  ],
  msg="You look down at your watch to see the time, its currently {}" current_time,
  }
