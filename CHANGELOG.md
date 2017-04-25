0.5
===
* Substrate configurations are now user-defined instead of hard-coded in the library. They are made available through the ```SubstrateConf``` struct which is passed to the construction function.
* The base ```System``` struct has been moved along with its constituent ```Residue``` and ```Atom``` structs to a new module. This is in preparation for making the library more general and not only for constructing substrates.
* The ```substrates``` module tests have been cleaned up. A couple of unit tests remain for very specific functionality, but tests for creating a substrate has been moved to the ```tests``` folder as proper integration tests.

0.4.2
=====
* Substrate residues can be shifted along z by a uniform random distribution
* This is added to the CLI utility as an option alongside an input position for the substrate

0.4.0
=====
* Library and CLI utility split into separate packages
* Silica and graphene substrates implemented
