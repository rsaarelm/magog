// Copyright (C) Risto Saarelma 2012

#ifndef WORLD_SPACE_ANIMS_HPP
#define WORLD_SPACE_ANIMS_HPP

#include "drawable.hpp"
#include "sprite.hpp"
#include <world/location.hpp>
#include <util/vec.hpp>
#include <map>
#include <set>
#include <queue>
#include <memory>

class World_Space_Anims {
public:
  void collect_sprites(const Vec2i& view_space_pos, std::set<Sprite>& output);

  void add(std::unique_ptr<Drawable> drawable, const Location& loc);
  void add(std::unique_ptr<Drawable> drawable, const Footprint& footprint);

  void update(float interval_sec);
private:
  typedef std::pair<std::unique_ptr<Drawable>, Footprint> Element;

  void remove(Element element);

  std::queue<Element> drawables;
  std::multimap<Location, std::pair<Vec2i, Drawable*>> locations;
};

#endif
