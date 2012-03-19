/* parts.hpp

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
#ifndef WORLD_PARTS_HPP
#define WORLD_PARTS_HPP

/// A proto-kind as a precursor to a full-fledged parts system.
class Blob_Part : public Part {
 public:
  static Kind s_get_kind() { return Blob_Kind; }

  Blob_Part() {}
  Blob_Part(Entity_Icon icon, int power, int armor, int damage, bool big=false)
    : icon(icon), power(power), energy(0), big(big), armor(armor), damage(damage) {}
  ~Blob_Part() {}

  virtual Kind get_kind() { return s_get_kind(); }
  Entity_Icon icon;
  int power;
  int energy;
  bool big; // XXX: Very crude, should have a more complex size system.

  int armor;
  int damage;
  Plain_Location loc;
 private:
  Blob_Part(const Blob_Part&);
  Blob_Part& operator=(const Blob_Part&);
};

#endif
