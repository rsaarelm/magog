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

// Given that there is a component system and lots of multi-entity interactions
// in play, when should we nevertheless implement operations as methdos of
// Entity. Basic rules of thumb, an operation should be a method of Entity if 1)
// most kinds of entities will use this method (exists and location would
// probably be good candidates) and 2) the method has an unambiguous single
// entity as it's main focus. Operation "attack" might not be a good method,
// since it's only of interest for the animate subset of entities, and it might
// also rather reliant on the combined properties of the attacker and the
// target entities.

class Entity {
 public:
  Entity(): uid(-1) {}

  Entity(const Entity& rhs) : uid(rhs.uid) {}
  Entity& operator=(const Entity& rhs) { uid = rhs.uid; }

  Entity(Entity_Id uid) : uid(uid) {}

  bool operator<(const Entity& rhs) const {
    return uid < rhs.uid;
  }

  bool operator==(const Entity& rhs) const {
    return uid == rhs.uid;
  }

  bool operator!=(const Entity& rhs) const {
    return uid != rhs.uid;
  }

  template <class T>
  T& _as() const;

  size_t hash() const {
    return uid;
  }

  Entity_Id id() const { return uid; }

 private:
  Entity_Id uid;
};

#endif
