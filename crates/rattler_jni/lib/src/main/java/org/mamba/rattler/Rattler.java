package org.mamba.rattler;

import java.util.List;
import java.util.Optional;

class Rattler {

    private static native boolean create(CreateOpts opts);

    static {
        System.loadLibrary("rattler_jni");
    }

    public static void main(String[] args) {
        create(new CreateOpts(List.of(args[0])));
    }
}