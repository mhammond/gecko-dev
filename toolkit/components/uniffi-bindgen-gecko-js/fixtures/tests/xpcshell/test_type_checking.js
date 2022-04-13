/* Any copyright is dedicated to the Public Domain.
   http://creativecommons.org/publicdomain/zero/1.0/ */

const Arithmetic = ChromeUtils.import(
  "resource://gre/modules/components-utils/Arithmetic.jsm"
);
const Geometry = ChromeUtils.import(
  "resource://gre/modules/components-utils/Geometry.jsm"
);

add_task(async function() {
  // Test our "type checking", which at this point is simply checking that
  // the correct number of arguments are passed

  await Assert.rejects(
    Arithmetic.add(2),
    /TypeError/,
    "add() call missing argument"
  );
  Assert.throws(
    () => Geometry.Point(0.0),
    /TypeError/,
    "Point constructor missing argument"
  );
});
