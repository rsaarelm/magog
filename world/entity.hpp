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

class Entities_System;

class Entity {
 public:
  Entity(): system(nullptr), uid(-1) {}

  Entity(const Entity& rhs) : system(rhs.system), uid(rhs.uid) {}
  Entity& operator=(const Entity& rhs) { system = rhs.system; uid = rhs.uid; }

  Entity(Entities_System* system, Entity_Id uid) : system(system), uid(uid) {}

  bool operator<(const Entity& rhs) const {
    ASSERT(system == rhs.system);
    return uid < rhs.uid;
  }

  bool operator==(const Entity& rhs) const {
    return uid == rhs.uid && system == rhs.system;
  }

  bool operator!=(const Entity& rhs) const {
    return uid != rhs.uid || system != rhs.system;
  }

  bool exists() const;

  template <class T>
  T& as() const;

  template <class T>
  bool has() const;

  void add_part(Part* new_part);

  size_t hash() const {
    return uid;
  }

  Entity_Id id() const { return uid; }

  Location location() const;

  /// Push the Entity into the ethereal void.
  void push();

  /// Checks if an entity in void can enter a location.
  bool can_pop(Location location) const;

  /// Pop the Entity back into existence from the void.
  void pop();

  /// Pop the Entity into a specific location.
  void pop(Location location);

  Footprint footprint(Location center) const;
  Footprint footprint() const;
 private:
  Entities_System* system;
  Entity_Id uid;
};

Part* _find_part(Entities_System* entities_system, Entity entity, Kind kind);

// XXX: Deprecated
Part* find_part(Entity entity, Kind kind);

template <class T>
T& Entity::as() const {
  // TODO: Assert system != nullptr, use system to find the part.
  Part* part = find_part(*this, T::s_get_kind());

  T* result = dynamic_cast<T*>(part);
  // If kind doesn't match to the actual object, there's been data corruption.
  ASSERT(result != nullptr);
  return *result;
}

template <class T>
bool Entity::has() const {
  try {
    Part* part = find_part(*this, T::s_get_kind());
    return true;
  } catch (Part_Not_Found& e) {
    return false;
  }
}

#endif
