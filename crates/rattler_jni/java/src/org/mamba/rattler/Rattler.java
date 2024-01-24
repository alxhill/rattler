package org.mamba.rattler;

import java.util.List;
import java.util.Optional;

class Rattler {

    public class RattlerCreate {
        private final List<String> channels;
        private final boolean dryRun;
        private final Optional<

        public RattlerCreate() {
            this.channels = List.of();
            this.dryRun = false;
            this.platform = Optional.empty();
        }


        public List<String> getChannels() {

        }
    }

    private static native void create(List<String> constraints);

    static {
        System.loadLibrary("rattler_jni");
    }

    public static void main(String[] args) {
        create(List.of(args[0]));
    }
}