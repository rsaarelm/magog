#include "actor.hpp"
#include "world.hpp"

/// Add a part to the Actor. Ownership of the part will move to Actor.
void Actor::add_part(Part* new_part) {
  ASSERT(xev::assoc_contains(World::get().actors, *this));
  // XXX: If old part is getting overwritten, does it need to be informed first?
  World::get().actors[*this][new_part->get_kind()] = std::unique_ptr<Part>(new_part);
}

Actor::operator bool() const {
  return actor_exists(*this);
}
