/* hud_system.hpp

   Copyright (C) 2012 Risto Saarelma

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#ifndef HUD_SYSTEM_HPP
#define HUD_SYSTEM_HPP

#include <world/entities_system.hpp>
#include <world/spatial_system.hpp>
#include <util/color.hpp>
#include <util/fonter_system.hpp>
#include <string>
#include <queue>
#include <list>

struct Message_String {
  std::string text;
  float time_read;
};

class Hud_System {
 public:
  Hud_System(
    Fonter_System& fonter,
    Entities_System& entities,
    Spatial_System& spatial)
    : fonter(fonter)
    , entities(entities)
    , spatial(spatial)
    , text_color("beige")
    , edge_color("black")
    , clock(0)
    , read_new_text_time(0)
    , letter_read_duration(0.2) {}

  void update(float interval_seconds);
  void draw();
  void add_msg(std::string str);
  void add_caption(std::string str);

  Color text_color;
  Color edge_color;
 private:
  void my_draw_text(const Vec2i& pos, const char* txt);

  Fonter_System& fonter;
  Entities_System& entities;
  Spatial_System& spatial;

  // Update the total time when texts will be read and return the time
  // the user should have read added_text.
  float time_read(std::string added_text);

  // Current time in seconds.
  float clock;
  // The estimated time when the user will have finished reading all the text
  // currently on screen. Either equal to clock or larger than it.
  float read_new_text_time;
  float letter_read_duration;
  std::list<Message_String> messages;
  std::queue<Message_String> captions;
};

#endif
