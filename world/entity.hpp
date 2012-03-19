/* entity.hpp

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

#ifndef WORLD_ENTITY_HPP
#define WORLD_ENTITY_HPP

#include <stdexcept>
#include <world/location.hpp>

typedef long Entity_Id;

typedef Entity_Id Entity;

class Entity_Exception : public std::exception {
};


/// Exception thrown when a UID has no corresponding Entity.
class Entity_Not_Found : public Entity_Exception {
 public:
  virtual const char* what() const throw() {
    return "Entity not found";
  }
};


/// Exception thrown when an Entity doesn't have an expected Part.
class Part_Not_Found : public Entity_Exception {
 public:
  virtual const char* what() const throw() {
    return "Part not found";
  }
};


enum Entity_Icon {
  icon_null,
  icon_infantry,
  icon_tank,
  icon_telos,
};

enum Kind {
  Blob_Kind,
  num_kinds
};


class Part {
 public:
  virtual ~Part() {}

  virtual Kind get_kind() = 0;
};

#endif
