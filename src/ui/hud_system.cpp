/* hud_system.cpp

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

#include "hud_system.hpp"
#include <ui/registry.hpp>
#include <world/parts.hpp>
#include <util/vec.hpp>
#include <util/game_loop.hpp>
#include <GL/gl.h>

using namespace std;

void Hud_System::update(float interval_seconds) {
  clock += interval_seconds;
  if (read_new_text_time < clock)
    read_new_text_time = clock;

  while (!messages.empty() && messages.front().time_read < clock) {
    messages.pop_front();
  }
  while (!captions.empty() && captions.front().time_read < clock) {
    captions.pop();
    if (!captions.empty())
      // Update the time-read of the new caption since it's becoming visible just now.
      captions.front().time_read = time_read(captions.front().text);
  }
  while (!events.empty() && events.front().time < clock) {
    events.front().fn();
    events.pop();
  }
}

void Hud_System::draw(Entity player) {
  Vec2f pos(0, 0);
  Vec2f offset(0, fonter.height());
  for (auto msg: messages) {
    // TODO: Support multiline messages
    my_draw_text(pos, msg.text.c_str());
    pos += offset;
  }

  if (!captions.empty()) {
    // TODO: Support multiline captions
    auto txt = captions.front().text.c_str();
    auto dim = Game_Loop::get().get_dim();
    // XXX: Recalculating TextWidth is expensive for every frame.
    my_draw_text(Vec2f(dim[0] / 2 - fonter.width(txt) / 2, 2 * dim[1] / 5), txt);
  }

  text_color.gl_color();

  // Print keyboard help
  for (int i = 0; i < 6; i++) {
    string chr = format("%s", "QWEASD"[i]);
    Vec2f pos(10 + 16 * (i % 3), Registry::window_h - 50 + 13 * (i / 3));
    fonter.draw(pos, Fonter_System::CENTER, chr.c_str());
  }

  for (int i = 0; i < 6; i++) {
    string chr = format("%s", "UIOJKL"[i]);
    Vec2f pos(Registry::window_w - 60 + 16 * (i % 3), Registry::window_h - 50 + 13 * (i / 3));
    fonter.draw(pos, Fonter_System::CENTER, chr.c_str());
  }

  // Draw the status line.
  if (player) {
    fonter.draw({0, Registry::window_h - 20.0f}, "Health: %s",
                entities.as<Blob_Part>(player).health);
  }
}

void Hud_System::add_msg(std::string str) {
  messages.push_back(Message_String{str, time_read(str)});
}

void Hud_System::add_caption(std::string str) {
  if (captions.empty())
    captions.push(Message_String{str, time_read(str)});
  else
    // Don't time_read pending captions, they'll be timed when they show up in
    // queue.
    captions.push(Message_String{str, 0});
}

void Hud_System::my_draw_text(const Vec2i& pos, const char* txt) {
  edge_color.gl_color();
  fonter.draw(pos + Vec2i(-1, 0), txt);
  fonter.draw(pos + Vec2i(0, -1), txt);
  fonter.draw(pos + Vec2i(1, 0), txt);
  fonter.draw(pos + Vec2i(0, 1), txt);
  text_color.gl_color();
  fonter.draw(pos, txt);
}

float Hud_System::time_read(std::string added_text) {
  float result = read_new_text_time + letter_read_duration * added_text.size();
  read_new_text_time = result;
  return result;
}

void Hud_System::add_event(float delay_seconds, std::function<void(void)> event_fn) {
  events.push(Event{clock + delay_seconds, event_fn});
}
