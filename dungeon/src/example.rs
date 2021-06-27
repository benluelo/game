// struct Something {
//     collection: Vec<u32>,
// }

// fn do_something(s: Something) {
//     // here, `s` is taken by value and is now owned by this function
//     // however, it is immutable as it is not declared as mut
//     println!("do_something: {:?}", s.collection);
// }

// fn do_something_mut(mut s: Something) {
//     // here, `s` is taken by value *mutably* and is now owned by this function
//     // it can be mutated within this function, and then it will be dropped when the
//     // frame for this function is dropped
//     s.collection.push(420);
//     println!("do_something_mut: {:?}", s.collection);
// }

// fn do_something_ref(s: &Something) {
//     // here, `s` is borrowed and is NOT owned by this function; the
//     // borrow of `s` will be dropped at the end of this function
//     // however, it is immutable as it is not declared as &mut
//     println!("do_something_ref: {:?}", s.collection);
// }

// fn do_something_ref_mut(s: &mut Something) {
//     // here, `s` is borrowed and is NOT owned by this function; the
//     // borrow of s will be dropped at the end of this function
//     // however, it is *mutable*; any changes made to `s` will be
//     // reflected at the call site
//     s.collection.push(12345);
//     println!("do_something_ref_mut: {:?}", s.collection);
// }

// fn main() {
//     {
//         let s = Something {
//             collection: vec![0, 1, 2, 3, 4, 5],
//         };

//         // s is moved into do_something here, so it can no longer be used
//         do_something(s);

//         // as such, this will error
//         // println!("{}", s.collection.len());
//     }

//     {
//         let s = Something {
//             collection: vec![0, 1, 2, 3, 4, 5],
//         };

//         // s is moved into do_something here, so it can no longer be used
//         do_something_mut(s);

//         // as such, this will error
//         // println!("{}", s.collection.len());
//     }

//     {
//         let s = Something {
//             collection: vec![0, 1, 2, 3, 4, 5],
//         };

//         // s is borrowed here; the current scope still owns s
//         do_something_ref(&s);

//         // this no longer errors
//         println!("length after do_something_ref: {}", s.collection.len());
//     }

//     {
//         // s must be declared as mutable, in order to take a mutable reference to it later
//         let mut s = Something {
//             collection: vec![0, 1, 2, 3, 4, 5],
//         };

//         // s is *mutably* borrowed here; the current scope still owns s
//         // do_something_ref_mut() is free to do whatever it wants with s now
//         do_something_ref_mut(&mut s);

//         // this, again, no longer errors, and will print 7 instead of 6
//         println!("length after do_something_ref_mut: {}", s.collection.len());
//     }
// }
