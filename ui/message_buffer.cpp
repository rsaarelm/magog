/* message_buffer.cpp

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

#include "message_buffer.hpp"
#include <GL/glew.h>
#include <util/vec.hpp>
#include <util/game_loop.hpp>

using namespace std;

void Message_Buffer::my_draw_text(const Vec2i& pos, const char* txt) {
  edge_color.gl_color();
  fonter.draw(pos + Vec2i(-1, 0), txt);
  fonter.draw(pos + Vec2i(0, -1), txt);
  fonter.draw(pos + Vec2i(1, 0), txt);
  fonter.draw(pos + Vec2i(0, 1), txt);
  text_color.gl_color();
  fonter.draw(pos, txt);
}

Message_Buffer::Message_Buffer(Fonter_System& fonter)
  : fonter(fonter)
  , text_color("white")
  , edge_color("black")
  , clock(0)
  , read_new_text_time(0)
  , letter_read_duration(0.2)
{}

void Message_Buffer::update(float interval_seconds) {
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
}

void Message_Buffer::draw() {
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
    my_draw_text(Vec2f(dim[0] / 2 - fonter.width(txt) / 2, dim[1] / 2), txt);
  }
}

void Message_Buffer::add_msg(std::string str) {
  messages.push_back(Message_String{str, time_read(str)});
}

void Message_Buffer::add_caption(std::string str) {
  if (captions.empty())
    captions.push(Message_String{str, time_read(str)});
  else
    // Don't time_read pending captions, they'll be timed when they show up in
    // queue.
    captions.push(Message_String{str, 0});
}

float Message_Buffer::time_read(std::string added_text) {
  float result = read_new_text_time + letter_read_duration * added_text.size();
  read_new_text_time = result;
  return result;
}
