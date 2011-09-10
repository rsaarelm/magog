USING: accessors arrays classes.tuple combinators dust.relation
dust.relation.private kernel literals tools.test ;

IN: dust.relation.relation-tests

TUPLE: book title author year ;

M: book obj>row { [ title>> ] [ author>> ] [ year>> ] } cleave 3array ;

CONSTANT: book1 T{ book f "Title1" "Author1" 1971 }
CONSTANT: book1.1 T{ book f "Title1" "Author1" 1973 }
CONSTANT: book2 T{ book f "Title2" "Author2" 1974 }
CONSTANT: book3 T{ book f "Title3" "Author1" 1974 }

[ t ] [ book1 obj>row { f f f } matches? ] unit-test
[ t ] [ book1 obj>row book1 obj>row matches? ] unit-test
[ f ] [ book1 obj>row book2 obj>row matches? ] unit-test

[ $ book1 ]
[ f f <relation>
  book1 over insert-tuple
  book1 over select-tuple nip ] unit-test

[ { $ book1 } ]
[ f f <relation>
  book1 over insert-tuple
  book1 over select-tuples nip ] unit-test

! Now test it with indexing on.
[ $ book1 ]
[ { 0 } f <relation>
  book1 over insert-tuple
  book1 over select-tuple nip ] unit-test

[ { $ book1.1 } ]
[ f 0 <relation>
  book1 over insert-tuple
  book1.1 over insert-tuple
  T{ book f "Title1" f f } over select-tuples nip ] unit-test
