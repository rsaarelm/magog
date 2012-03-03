#include <util/vec.hpp>
#include <util/mtx.hpp>
#define BOOST_TEST_MODULE vec
#include <boost/test/unit_test.hpp>

BOOST_AUTO_TEST_CASE( vec1 ) {
  BOOST_CHECK((Vec<double, 3>{1.0, 2.0, 3.0}) ==
              (Vec<double, 3>{1.0, 2.0, 3.0}));

  auto mtx = Mtx<int, 2, 3>{
    1, 2,
    3, 4,
    5, 6};
  BOOST_CHECK(mtx[1] == (Vec<int, 3>{2, 4, 6}));
  BOOST_CHECK(mtx[1][1] == 4);

  // Complex product
  BOOST_CHECK((Vec<double, 2>{3, 2} * Vec<double, 2>{1, 4} ==
               Vec<double, 2>{-5, 14}));

  // Cross product.
  BOOST_CHECK((Vec<double, 3>{3, -3, 1} * Vec<double, 3>{4, 9, 2} ==
               Vec<double, 3>{-15, -2, 39}));

  // Quaternion product.
  BOOST_CHECK((Vec<double, 4>{1, 0, 1, 0} * Vec<double, 4>{1, 0.5, 0.5, 0.75} ==
              Vec<double, 4>{0.5, 1.25, 1.5, 0.25}));

  // Elementwise multiplication using conversion to diagonal matrix (inefficient).
  BOOST_CHECK((Vec<double, 3>{1, 2, 3}.as_diagonal() * Vec<double, 3>{4, 5, 6} ==
               Vec<double, 3>{4, 10, 18}));
}
