/* effects.cpp

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

#include <world/effects.hpp>
#include <ui/drawable.hpp>
#include <ui/game_screen.hpp>
#include <memory>


struct BeamDrawable : public Drawable {
  BeamDrawable(const Vec2i& dir, int length, const Color& color = Color("pink"), float life = 0.2)
    : dir(dir), length(length), color(color), life(life) {}

  virtual Footprint footprint(const Location& start) const {
    Footprint result;
    Location current_loc = start;
    Vec2i offset = Vec2i(0, 0);

    for (int i = 0; i < length; i++) {
      result[offset] = current_loc;
      offset = offset + dir;
      current_loc = current_loc.offset_and_portal(dir);
    }
    return result;
  }

  virtual bool update(float interval_sec) {
    life -= interval_sec;
    return life > 0;
  }

  virtual void draw(const Vec2f& offset) {
    Vec2f start = offset + Vec2f(tile_size / 2, tile_size / 2);
    Vec2f end = tile_projection * Vec2f(dir * length) + offset +
      Vec2f(tile_size / 2, tile_size / 2);
    glBindTexture(GL_TEXTURE_2D, 0);
    color.gl_color();
    glBegin(GL_LINES);
    glVertex2f(start[0], start[1]);
    glVertex2f(end[0], end[1]);
    glEnd();
  }

  virtual int get_z_layer() const { return 100; }

  Vec2i dir;
  int length;
  Color color;
  float life;
};


static Game_Screen* get_game_screen() {
  Game_State* state = Game_Loop::get().top_state();
  return dynamic_cast<Game_Screen*>(state);
}

void raw_msg(std::string str) {
  Game_Screen* scr = get_game_screen();
  if (scr) {
    scr->msg_buffer.add_msg(str);
  }
}

void beam_fx(const Location& location, const Vec2i& dir, int length, const Color& color) {
  Game_Screen* scr = get_game_screen();
  if (scr) {
    scr->world_anims.add(
      std::unique_ptr<Drawable>(new BeamDrawable(dir, length, color)),
      location);
  }
}

