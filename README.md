Factorio Circuits Language
==========================

FCL is a programming language and compiler, for the Factorio circuit network.

This project is created as a proof-of-concept, and to create more advanced circuits inside Factorio, in a maintainable way.

[Feel free](https://github.com/Discookie/factorio-simple-circuit-creator/issues) to shoot me ideas about the language, or the project!

Usage
-----

For now, it reads the program from the standard input, for example:  
``cat example-input.txt | cargo run``

Example code can be found in the file ``example-input.txt``.

License
-------

While the code in this repository is licensed under the MIT license, a dependency, [factorio-blueprint](https://github.com/coriolinus/factorio-blueprint), is licensed under the GNU GPL.
This is not a problem for now, but in the future it will be replaced with a more fine-tuned system for creating blueprints.
