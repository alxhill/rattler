#!/usr/bin/env bash

javac Rattler.java
java -cp . -Djava.library.path=/Users/alxhill/dev/rust/rattler/target/debug Rattler hello
