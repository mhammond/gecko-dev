/* Any copyright is dedicated to the Public Domain.
   http://creativecommons.org/publicdomain/zero/1.0/ */

const Rondpoint = ChromeUtils.import("resource://gre/modules/components-utils/Rondpoint.jsm");

const { Enumeration, EnumerationAvecDonnees, copieEnumeration, copieEnumerations, copieCarte } = Rondpoint;

add_task(async function() {
   const un = Enumeration.UN;
   const deux = Enumeration.DEUX;
   const enumeration = await copieEnumeration(un);
   const enumerations = await copieEnumerations([un, deux]);
   const zero = new EnumerationAvecDonnees.Zero();
   const obj = {
      "zero": zero
   };

   const ret = await copieCarte(obj);
   Assert.equal(enumeration, un);
   Assert.ok(enumeration === Enumeration.UN);
   Assert.deepEqual(obj, ret);
   Assert.deepEqual(enumerations, [un, deux]);
   Assert.ok(ret.zero instanceof EnumerationAvecDonnees.Zero)
   Assert.ok(ret.zero instanceof EnumerationAvecDonnees)
})

Assert.ok(true);
