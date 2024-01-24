import java.util.List;

class Rattler {
    private static native void create(List<String> constraints);

    static {
        System.loadLibrary("rattler_jni");
    }

    public static void main(String[] args) {
        create(List.of(args[0]));
    }
}