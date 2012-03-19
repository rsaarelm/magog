/* ui_fx_system.hpp

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
#ifndef UI_UI_FX_SYSTEM_HPP
#define UI_UI_FX_SYSTEM_HPP

#include <world/fx_system.hpp>

class Ui_Fx_System : public Fx_System {
  virtual void beam(Location location, const Vec2i& dir, int length, const Color& color);

  virtual void explosion(Location location, int intensity, const Color& color);

private:
  virtual void raw_msg(std::string text);
};

#endif
