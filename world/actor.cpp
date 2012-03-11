/* actor.cpp

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

#include "actor.hpp"
#include "world.hpp"

/// Add a part to the Actor. Ownership of the part will move to Actor.
void Actor::add_part(Part* new_part) {
  ASSERT(assoc_contains(World::get().actors, *this));
  // XXX: If old part is getting overwritten, does it need to be informed first?
  World::get().actors[*this][new_part->get_kind()] = std::unique_ptr<Part>(new_part);
}

bool Actor::exists() const {
  return actor_exists(*this);
}
