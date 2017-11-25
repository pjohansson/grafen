0.9
===
* Rewrite of the object construction framework. Instead of a messy matching to different object types using `IntoComponent`, the simple trait `Component` has been redone to yield all necessary information from objects.
* In particular `Component` prescribed the method `iter_atoms` which yields all atoms in a component with their indices as required for output to disk.
* Objects are added to the `DataBase` using a static dispatcher enum as a wrapper. This wrapper lets all objects exist in one `Vec`. It is created using the new macro `create_entry_wrapper` which passes through all the required implementations.
* Object definitions now contain all the information required to process and use them. Instead of containing as little information as possible and then have a separate object wrap them with more methods for `DataBase` use, etc. This second layer is unnecessarily confusing (although we still need one separate layer for the static dispatch, but as per above this is fully transparent).
* The `ResidueBase` object has been renamed to `Residue`.
* `System` has been moved to `system` and improved to user proper methods for yielding information for output. This includes implementing `iter_atoms` through the `Component` trait which links to all components` methods of that name and merges them to a single whole.

0.9.1
-----
* Add the ability to edit constructed components by translating them or cutting coordinates from them using another constructed component.
* Sheets are now constructed with periodic boundary conditions being applied. Residue coordinates will not extend beyond the component box size (although atoms belonging to the residue might).

0.8
===
* The user interface has been reworked, using `dialoguer` instead of plain text selection.

0.7
===
* The command line interface has been reworked to allow for new types of objects.
* This includes the database being able to store these on disk.
* In particular, a `Cylinder` object has been added. It is a sheet of molecules, folded into a cylinder. Practically it represents (eg.) a nanotube.
* `Component`s now own their `ResidueBase`. As such the `Residue` object has been removed.

0.7.1
-----
* `Cylinder`s can now be created as filled volumes, not only sheets.

0.6
===
* Substrate definitions are now read from a database at runtime. This database is JSON formatted and implemented using the `serde` framework.
* The binary has been updated to construct systems as read from this database. This is an early implementation of the binary, it is in need of improvement.

0.6.1
-----
* It is now possible to properly edit the database by adding or removing residue and substrate definitions.
* A database can be saved and moved to different locations.
* The GROMOS file output has been changed to number atoms not per-residue but by their absolute index in the system.

0.5
===
* Substrate configurations are now user-defined instead of hard-coded in the library. They are made available through the `SubstrateConf` struct which is passed to the construction function.
* The base `System` struct has been moved along with its constituent `Residue` and `Atom` structs to a new module. This is in preparation for making the library more general and not only for constructing substrates.
* The `substrates` module tests have been cleaned up. A couple of unit tests remain for very specific functionality, but tests for creating a substrate has been moved to the `tests` folder as proper integration tests.

0.5.1
-----
* The `substrates` module has been renamed to the singular `substrate`.
* The `lattice` module is now a *private* submodule to `substrate`.
* The `output` module has been moved to the binary.
* A macro `resbase` has been added to easily construct `ResidueBase` objects.
* `ResidueBase` no longer implements methods for getting a silica or graphene base. These definitions belong in the binary, not in the library.

0.5.2
-----
* Implement a Poisson Disc distribution for generating substrates.

0.4
===
* Library and CLI utility split into separate packages
* Silica and graphene substrates implemented

0.4.1
-----
No notable changes.

0.4.2
-----
* Substrate residues can be shifted along z by a uniform random distribution
* This is added to the CLI utility as an option alongside an input position for the substrate
