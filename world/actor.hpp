// Copyright (C) 2012 Risto Saarelma

#ifndef WORLD_ACTOR_HPP
#define WORLD_ACTOR_HPP

#include <stdexcept>

typedef long Actor_Id;


class Actor_Exception : public std::exception {
};


/// Exception thrown when a UID has no corresponding Actor.
class Actor_Not_Found : public Actor_Exception {
 public:
  virtual const char* what() const throw() {
    return "Actor not found";
  }
};


/// Exception thrown when an Actor doesn't have an expected Part.
class Part_Not_Found : public Actor_Exception {
 public:
  virtual const char* what() const throw() {
    return "Part not found";
  }
};


enum Actor_Icon {
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


class Actor {
 public:
  Actor(): uid(-1) {}
  Actor(const Actor& rhs) : uid(rhs.uid) {}
  Actor(Actor_Id uid) : uid(uid) {}

  bool operator<(const Actor& rhs) const {
    return uid < rhs.uid;
  }

  bool operator==(const Actor& rhs) const {
    return uid == rhs.uid;
  }

  bool operator!=(const Actor& rhs) const {
    return uid != rhs.uid;
  }

  /// Return whether the Actor actually exists in world.
  operator bool() const;

  template <class T>
  T& as();

  void add_part(Part* new_part);

  size_t hash() const {
    return uid;
  }

  Actor_Id id() const { return uid; }
 private:
  Actor_Id uid;
};

#endif
