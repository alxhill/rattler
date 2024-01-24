use jni::JNIEnv;

use jni::objects::{JClass, JObject};

#[no_mangle]
pub extern "system" fn Java_Rattler_create<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
    input: JObject<'local>,
) {
    dbg!(env);
    dbg!(class);
    dbg!(input);
}
