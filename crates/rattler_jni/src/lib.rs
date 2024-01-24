use jni::JNIEnv;

use jni::objects::{JClass, JObject};

#[no_mangle]
pub extern "system" fn Java_org_mamba_rattler_Rattler_create<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
    input: JObject<'local>,
) {
    let specs: JObject<'local> = env
        .call_method(&input, "getSpecs", "()Ljava/util/List;", &[])
        .unwrap()
        .l()
        .unwrap();
    let spec_list = env.get_list(&specs).unwrap();

    let mut iterator = spec_list.iter(&mut env).unwrap();
    while let Some(obj) = iterator.next(&mut env).unwrap() {
        let str: String = env.get_string((&obj).into()).unwrap().into();
        println!("{}", str);
    }
}

/*
public class org.mamba.rattler.CreateOpts {
  public org.mamba.rattler.CreateOpts(java.util.List<java.lang.String>);
    descriptor: (Ljava/util/List;)V

  public org.mamba.rattler.CreateOpts(java.util.List<java.lang.String>, java.util.List<java.lang.String>, boolean, java.util.Optional<java.lang.String>, java.util.List<java.lang.String>);
    descriptor: (Ljava/util/List;Ljava/util/List;ZLjava/util/Optional;Ljava/util/List;)V

  public java.util.List<java.lang.String> getChannels();
    descriptor: ()Ljava/util/List;

  public java.util.List<java.lang.String> getSpecs();
    descriptor: ()Ljava/util/List;

  public boolean isDryRun();
    descriptor: ()Z

  public java.util.Optional<java.lang.String> getPlatform();
    descriptor: ()Ljava/util/Optional;

  public java.util.List<java.lang.String> getVirtualPackage();
    descriptor: ()Ljava/util/List;
}
 */
