#ifndef UTIL_ALG_HPP
#define UTIL_ALG_HPP

/// \file alg.hpp \brief Generic helper algorithms

#include <functional>
#include <algorithm>
#include <boost/range.hpp>
#include <boost/range/any_range.hpp>
#include <boost/optional.hpp>

/**
 * Experimental general default range construct.
 *
 * Wrote this because I couldn't figure out how to nicely write return types for
 * functions that return Boost ranges otherwise. Uses the somewhat inefficient
 * `boost::any_range`.
 *
 * While we're waiting for gcc to get template aliasing working, will have to
 * make do with the awkward struct typedef idiom for getting the partially
 * specialized type.
 */
template<class C>
struct Range {
  typedef boost::any_range<C, boost::single_pass_traversal_tag, C, std::ptrdiff_t> T;
};

/// Helper function to run all_of over a range without spelling out the begin/end.
template<class Range, class Unary_Predicate>
bool all_of(const Range& a, Unary_Predicate p) {
  return std::all_of(a.begin(), a.end(), p);
}

/**
 * Return whether p holds for each corresponding pair of elements from ranges a and b.
 *
 * If a and b have different lengths, the elements that have no pair are
 * ignored.
 */
template<class Range, class Binary_Predicate>
bool pairwise_all_of(const Range& a, const Range& b, Binary_Predicate p) {
  auto a1 = a.begin(), a2 = a.end();
  auto b1 = b.begin(), b2 = b.end();
  while (a1 != a2 && b1 != b2 && p(*a1, *b1)) {
    ++a1;
    ++b1;
  }
  return a1 == a2 || b1 == b2;
}

/// Convenience function for looking up assoc values that might not be present.
template<class Assoc>
boost::optional<typename Assoc::mapped_type> assoc_find(
    Assoc& assoc, const typename Assoc::key_type& key) {
  auto iter = assoc.find(key);
  if (iter != assoc.end())
    return boost::optional<typename Assoc::mapped_type>(iter->second);
  return boost::optional<typename Assoc::mapped_type>();
}

/// Convenience function for returning an assoc value or a not-found alternative value.
template<class Assoc>
typename Assoc::mapped_type assoc_find_or(
    Assoc& assoc,
    const typename Assoc::key_type& key,
    const typename Assoc::mapped_type& not_found_value) {
  boost::optional<typename Assoc::mapped_type> result;
  auto iter = assoc.find(key);
  if (iter != assoc.end())
    return iter->second;
  return not_found_value;
}

/// Convenience function for seeing if an assoc contains a value.
template<class Assoc>
bool assoc_contains(const Assoc& assoc, const typename Assoc::key_type& key) {
  return assoc.find(key) != assoc.end();
}

#endif
