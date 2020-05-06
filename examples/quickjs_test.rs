extern crate rusty_pipe;
use quick_js::{Context, JsValue};

fn main(){
    let context = Context::new().unwrap();
    let r = context.eval("function sum(a,b){return a+b;} sum(2,5)");
    println!("{:?}",r);

    let r2 = context.eval(r##"
var Fv={
    Cc:function(a,b){a.splice(0,b)},
    UF:function(a){a.reverse()},
    Vw:function(a,b){
        var c=a[0];
        a[0]=a[b%a.length];
        a[b%a.length]=c
    }
};
var Gv=function(a){
    a=a.split("");
    Fv.Vw(a,25);
    Fv.Vw(a,47);
    Fv.UF(a,63);
    Fv.Cc(a,1);
    Fv.Vw(a,47);
    return a.join("")
};
function decrypt(a){return Gv(a);};decrypt("M=AOzsJhJF_0tQ657YzWo2o_dI4GAFEjmi2l6f1uml7e3CQ=C0qfVIBJA13JjabpXp9nsd14cLOQa8i0nZY3ZlYoU2wQgIQRwsLlPpJCC")
    "##);

    println!("{:?}", r2);
}