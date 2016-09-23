use std::hash::Hash;
use std::collections::HashMap;

use core::Message;
use core::ProtobufEnum;
use core::message_down_cast;
use reflect::EnumValueDescriptor;
use types::*;

use repeated::RepeatedField;

use super::map::ReflectMap;
use super::repeated::ReflectRepeated;
use super::ReflectFieldRef;


/// this trait should not be used directly, use `FieldDescriptor` instead
pub trait FieldAccessor {
    fn name_generic(&self) -> &'static str;
    fn has_field_generic(&self, m: &Message) -> bool;
    fn len_field_generic(&self, m: &Message) -> usize;
    fn get_message_generic<'a>(&self, m: &'a Message) -> &'a Message;
    fn get_rep_message_item_generic<'a>(&self, m: &'a Message, index: usize) -> &'a Message;
    fn get_enum_generic(&self, m: &Message) -> &'static EnumValueDescriptor;
    fn get_rep_enum_item_generic(&self, m: &Message, index: usize) -> &'static EnumValueDescriptor;
    fn get_str_generic<'a>(&self, m: &'a Message) -> &'a str;
    fn get_rep_str_generic<'a>(&self, m: &'a Message) -> &'a [String];
    fn get_bytes_generic<'a>(&self, m: &'a Message) -> &'a [u8];
    fn get_rep_bytes_generic<'a>(&self, m: &'a Message) -> &'a [Vec<u8>];
    fn get_u32_generic(&self, m: &Message) -> u32;
    fn get_rep_u32_generic<'a>(&self, m: &'a Message) -> &'a [u32];
    fn get_u64_generic(&self, m: &Message) -> u64;
    fn get_rep_u64_generic<'a>(&self, m: &'a Message) -> &'a [u64];
    fn get_i32_generic(&self, m: &Message) -> i32;
    fn get_rep_i32_generic<'a>(&self, m: &'a Message) -> &'a [i32];
    fn get_i64_generic(&self, m: &Message) -> i64;
    fn get_rep_i64_generic<'a>(&self, m: &'a Message) -> &'a [i64];
    fn get_bool_generic(&self, m: &Message) -> bool;
    fn get_rep_bool_generic<'a>(&self, m: &'a Message) -> &'a [bool];
    fn get_f32_generic(&self, m: &Message) -> f32;
    fn get_rep_f32_generic<'a>(&self, m: &'a Message) -> &'a [f32];
    fn get_f64_generic(&self, m: &Message) -> f64;
    fn get_rep_f64_generic<'a>(&self, m: &'a Message) -> &'a [f64];

    fn get_reflect<'a>(&self, m: &'a Message) -> ReflectFieldRef<'a>;
}


trait GetSingularMessage<M> {
    fn get_message<'a>(&self, m: &'a M) -> &'a Message;
}

struct GetSingularMessageImpl<M, N> {
    get: for<'a> fn(&'a M) -> &'a N,
}

impl<M : Message, N : Message + 'static> GetSingularMessage<M> for GetSingularMessageImpl<M, N> {
    fn get_message<'a>(&self, m: &'a M) -> &'a Message {
        (self.get)(m)
    }
}


trait GetSingularEnum<M> {
    fn get_enum(&self, m: &M) -> &'static EnumValueDescriptor;
}

struct GetSingularEnumImpl<M, E> {
    get: fn(&M) -> E,
}

impl<M : Message, E : ProtobufEnum> GetSingularEnum<M> for GetSingularEnumImpl<M, E> {
    fn get_enum(&self, m: &M) -> &'static EnumValueDescriptor {
        (self.get)(m).descriptor()
    }
}


trait GetRepeatedMessage<M> {
    fn len_field(&self, m: &M) -> usize;
    fn get_message_item<'a>(&self, m: &'a M, index: usize) -> &'a Message;
}

struct GetRepeatedMessageImpl<M, N> {
    get: for<'a> fn(&'a M) -> &'a [N],
}

impl<M : Message, N : Message + 'static> GetRepeatedMessage<M> for GetRepeatedMessageImpl<M, N> {
    fn len_field(&self, m: &M) -> usize {
        (self.get)(m).len()
    }

    fn get_message_item<'a>(&self, m: &'a M, index: usize) -> &'a Message {
        &(self.get)(m)[index]
    }
}


trait GetRepeatedEnum<M> {
    fn len_field(&self, m: &M) -> usize;
    fn get_enum_item(&self, m: &M, index: usize) -> &'static EnumValueDescriptor;
}

struct GetRepeatedEnumImpl<M, E> {
    get: for<'a> fn(&'a M) -> &'a [E],
}

impl<M : Message, E : ProtobufEnum> GetRepeatedEnum<M> for GetRepeatedEnumImpl<M, E> {
    fn len_field(&self, m: &M) -> usize {
        (self.get)(m).len()
    }

    fn get_enum_item(&self, m: &M, index: usize) -> &'static EnumValueDescriptor {
        (self.get)(m)[index].descriptor()
    }
}


enum SingularGet<M> {
    U32(fn(&M) -> u32),
    U64(fn(&M) -> u64),
    I32(fn(&M) -> i32),
    I64(fn(&M) -> i64),
    F32(fn(&M) -> f32),
    F64(fn(&M) -> f64),
    Bool(fn(&M) -> bool),
    String(for<'a> fn(&'a M) -> &'a str),
    Bytes(for<'a> fn(&'a M) -> &'a [u8]),
    Enum(Box<GetSingularEnum<M> + 'static>),
    Message(Box<GetSingularMessage<M> + 'static>),
}

enum RepeatedGet<M> {
    U32(for<'a> fn(&'a M) -> &'a [u32]),
    U64(for<'a> fn(&'a M) -> &'a [u64]),
    I32(for<'a> fn(&'a M) -> &'a [i32]),
    I64(for<'a> fn(&'a M) -> &'a [i64]),
    F32(for<'a> fn(&'a M) -> &'a [f32]),
    F64(for<'a> fn(&'a M) -> &'a [f64]),
    Bool(for<'a> fn(&'a M) -> &'a [bool]),
    String(for<'a> fn(&'a M) -> &'a [String]),
    Bytes(for<'a> fn(&'a M) -> &'a [Vec<u8>]),
    Enum(Box<GetRepeatedEnum<M> + 'static>),
    Message(Box<GetRepeatedMessage<M> + 'static>),
}

impl<M : Message> RepeatedGet<M> {
    fn len_field(&self, m: &M) -> usize {
        match *self {
            RepeatedGet::U32(get) => get(m).len(),
            RepeatedGet::U64(get) => get(m).len(),
            RepeatedGet::I32(get) => get(m).len(),
            RepeatedGet::I64(get) => get(m).len(),
            RepeatedGet::F32(get) => get(m).len(),
            RepeatedGet::F64(get) => get(m).len(),
            RepeatedGet::Bool(get) => get(m).len(),
            RepeatedGet::String(get) => get(m).len(),
            RepeatedGet::Bytes(get) => get(m).len(),
            RepeatedGet::Enum(ref get) => get.len_field(m),
            RepeatedGet::Message(ref get) => get.len_field(m),
        }
    }
}

trait FieldAccessor2<M, R: ?Sized>
    where
        M : Message + 'static,
{
    fn get_field<'a>(&self, &'a M) -> &'a R;
    fn mut_field<'a>(&self, &'a mut M) -> &'a mut R;
}

enum FieldAccessorFunctions<M> {
    Singular { has: fn(&M) -> bool, get: SingularGet<M> },
    Repeated(RepeatedGet<M>),
    Map(Box<FieldAccessor2<M, ReflectMap>>),
    RepeatedField(Box<FieldAccessor2<M, ReflectRepeated>>),
}


struct FieldAccessorImpl<M> {
    name: &'static str,
    fns: FieldAccessorFunctions<M>,
}

impl<M : Message + 'static> FieldAccessor for FieldAccessorImpl<M> {
    fn name_generic(&self) -> &'static str {
        self.name
    }

    fn has_field_generic(&self, m: &Message) -> bool {
        match self.fns {
            FieldAccessorFunctions::Singular { has, .. } => has(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn len_field_generic(&self, m: &Message) -> usize {
        match self.fns {
            FieldAccessorFunctions::Repeated(ref r) => r.len_field(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_message_generic<'a>(&self, m: &'a Message) -> &'a Message {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::Message(ref get), .. } =>
                get.get_message(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_enum_generic(&self, m: &Message) -> &'static EnumValueDescriptor {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::Enum(ref get), .. } =>
                get.get_enum(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_str_generic<'a>(&self, m: &'a Message) -> &'a str {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::String(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_bytes_generic<'a>(&self, m: &'a Message) -> &'a [u8] {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::Bytes(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_u32_generic(&self, m: &Message) -> u32 {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::U32(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_u64_generic(&self, m: &Message) -> u64 {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::U64(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_i32_generic(&self, m: &Message) -> i32 {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::I32(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_i64_generic(&self, m: &Message) -> i64 {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::I64(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_f32_generic(&self, m: &Message) -> f32 {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::F32(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_f64_generic(&self, m: &Message) -> f64 {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::F64(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_bool_generic(&self, m: &Message) -> bool {
        match self.fns {
            FieldAccessorFunctions::Singular { get: SingularGet::Bool(get), .. } =>
                get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_message_item_generic<'a>(&self, m: &'a Message, index: usize) -> &'a Message {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::Message(ref get)) =>
                get.get_message_item(message_down_cast(m), index),
            _ => panic!(),
        }
    }

    fn get_rep_enum_item_generic(&self, m: &Message, index: usize) -> &'static EnumValueDescriptor {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::Enum(ref get)) =>
                get.get_enum_item(message_down_cast(m), index),
            _ => panic!(),
        }
    }

    fn get_rep_str_generic<'a>(&self, m: &'a Message) -> &'a [String] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::String(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_bytes_generic<'a>(&self, m: &'a Message) -> &'a [Vec<u8>] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::Bytes(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_u32_generic<'a>(&self, m: &'a Message) -> &'a [u32] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::U32(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_u64_generic<'a>(&self, m: &'a Message) -> &'a [u64] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::U64(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_i32_generic<'a>(&self, m: &'a Message) -> &'a [i32] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::I32(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_i64_generic<'a>(&self, m: &'a Message) -> &'a [i64] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::I64(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_f32_generic<'a>(&self, m: &'a Message) -> &'a [f32] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::F32(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_f64_generic<'a>(&self, m: &'a Message) -> &'a [f64] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::F64(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_rep_bool_generic<'a>(&self, m: &'a Message) -> &'a [bool] {
        match self.fns {
            FieldAccessorFunctions::Repeated(RepeatedGet::Bool(get)) => get(message_down_cast(m)),
            _ => panic!(),
        }
    }

    fn get_reflect<'a>(&self, m: &'a Message) -> ReflectFieldRef<'a> {
        match self.fns {
            FieldAccessorFunctions::RepeatedField(ref accessor2) =>
                ReflectFieldRef::Repeated(accessor2.get_field(message_down_cast(m))),
            FieldAccessorFunctions::Map(ref accessor2) =>
                ReflectFieldRef::Map(accessor2.get_field(message_down_cast(m))),
            _ =>
                ReflectFieldRef::Other,
        }
    }
}


// singular

pub fn make_singular_u32_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> u32,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::U32(get),
        },
    })
}

fn const_true<T>(_: &T) -> bool {
    true
}

pub fn make_singular_proto3_u32_accessor<M : Message + 'static>(
        name: &'static str,
        get: fn(&M) -> u32,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::U32(get),
        },
    })
}

pub fn make_singular_i32_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> i32,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::I32(get),
        },
    })
}

pub fn make_singular_proto3_i32_accessor<M : Message + 'static>(
        name: &'static str,
        get: fn(&M) -> i32,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::I32(get),
        },
    })
}

pub fn make_singular_u64_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> u64,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::U64(get),
        },
    })
}

pub fn make_singular_proto3_u64_accessor<M : Message + 'static>(
        name: &'static str,
        get: fn(&M) -> u64,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::U64(get),
        },
    })
}

pub fn make_singular_i64_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> i64,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::I64(get),
        },
    })
}

pub fn make_singular_proto3_i64_accessor<M : Message + 'static>(
        name: &'static str,
        get: fn(&M) -> i64,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::I64(get),
        },
    })
}

pub fn make_singular_f32_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> f32,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::F32(get),
        },
    })
}

pub fn make_singular_proto3_f32_accessor<M : Message + 'static>(
        name: &'static str,
        get: fn(&M) -> f32,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::F32(get),
        },
    })
}

pub fn make_singular_f64_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> f64,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::F64(get),
        },
    })
}

pub fn make_singular_proto3_f64_accessor<M : Message + 'static>(
        name: &'static str,
        get: fn(&M) -> f64,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::F64(get),
        },
    })
}

pub fn make_singular_bool_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> bool,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::Bool(get),
        },
    })
}

pub fn make_singular_proto3_bool_accessor<M : Message + 'static>(
        name: &'static str,
        get: fn(&M) -> bool,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::Bool(get),
        },
    })
}

pub fn make_singular_enum_accessor<M : Message + 'static, E : ProtobufEnum + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: fn(&M) -> E,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::Enum(
                Box::new(GetSingularEnumImpl { get: get }),
            ),
        },
    })
}

pub fn make_singular_proto3_enum_accessor<M : Message + 'static, E : ProtobufEnum + 'static>(
        name: &'static str,
        get: fn(&M) -> E,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::Enum(
                Box::new(GetSingularEnumImpl { get: get }),
            ),
        },
    })
}

pub fn make_singular_string_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: for<'a> fn(&'a M) -> &'a str,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::String(get),
        },
    })
}

pub fn make_singular_proto3_string_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a str,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::String(get),
        },
    })
}

pub fn make_singular_bytes_accessor<M : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: for<'a> fn(&'a M) -> &'a [u8],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::Bytes(get),
        },
    })
}

pub fn make_singular_proto3_bytes_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [u8],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: const_true,
            get: SingularGet::Bytes(get),
        },
    })
}

pub fn make_singular_message_accessor<M : Message + 'static, F : Message + 'static>(
        name: &'static str,
        has: fn(&M) -> bool,
        get: for<'a> fn(&'a M) -> &'a F,
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Singular {
            has: has,
            get: SingularGet::Message(
                Box::new(GetSingularMessageImpl { get: get }),
            ),
        },
    })
}

// repeated

pub fn make_repeated_u32_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [u32],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::U32(get)),
    })
}

pub fn make_repeated_i32_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [i32],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::I32(get)),
    })
}

pub fn make_repeated_u64_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [u64],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::U64(get)),
    })
}

pub fn make_repeated_i64_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [i64],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::I64(get)),
    })
}

pub fn make_repeated_f32_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [f32],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::F32(get)),
    })
}

pub fn make_repeated_f64_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [f64],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::F64(get)),
    })
}

pub fn make_repeated_bool_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [bool],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::Bool(get)),
    })
}

pub fn make_repeated_string_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [String],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::String(get)),
    })
}

pub fn make_repeated_bytes_accessor<M : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [Vec<u8>],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::Bytes(get)),
    })
}

pub fn make_repeated_enum_accessor<M : Message + 'static, E : ProtobufEnum + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [E],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::Enum(
            Box::new(GetRepeatedEnumImpl { get: get }),
        )),
    })
}

pub fn make_repeated_message_accessor<M : Message + 'static, F : Message + 'static>(
        name: &'static str,
        get: for<'a> fn(&'a M) -> &'a [F],
    ) -> Box<FieldAccessor + 'static>
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Repeated(RepeatedGet::Message(
            Box::new(GetRepeatedMessageImpl { get: get }),
        )),
    })
}



struct VecAccessor<M, V>
    where
        M : Message + 'static,
        V : ProtobufType,
{
    get_vec: for<'a> fn(&'a M) -> &'a Vec<V::Value>,
    mut_vec: for<'a> fn(&'a mut M) -> &'a mut Vec<V::Value>,
}

impl<M, V> FieldAccessor2<M, ReflectRepeated> for VecAccessor<M, V>
    where
        M : Message + 'static,
        V : ProtobufType,
{
    fn get_field<'a>(&self, m: &'a M) -> &'a ReflectRepeated {
        (self.get_vec)(m) as &ReflectRepeated
    }

    fn mut_field<'a>(&self, m: &'a mut M) -> &'a mut ReflectRepeated {
        (self.mut_vec)(m) as &mut ReflectRepeated
    }
}



pub fn make_vec_accessor<M, V>(
    name: &'static str,
    get_vec: for<'a> fn(&'a M) -> &'a Vec<V::Value>,
    mut_vec: for<'a> fn(&'a mut M) -> &'a mut Vec<V::Value>)
        -> Box<FieldAccessor + 'static>
    where
        M : Message + 'static,
        V : ProtobufType + 'static,
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::RepeatedField(Box::new(VecAccessor::<M, V> {
            get_vec: get_vec,
            mut_vec: mut_vec,
        })),
    })
}


struct RepeatedFieldAccessor<M, V>
    where
        M : Message + 'static,
        V : ProtobufType,
{
    get_vec: for<'a> fn(&'a M) -> &'a RepeatedField<V::Value>,
    mut_vec: for<'a> fn(&'a mut M) -> &'a mut RepeatedField<V::Value>,
}

impl<M, V> FieldAccessor2<M, ReflectRepeated> for RepeatedFieldAccessor<M, V>
    where
        M : Message + 'static,
        V : ProtobufType,
{
    fn get_field<'a>(&self, m: &'a M) -> &'a ReflectRepeated {
        (self.get_vec)(m) as &ReflectRepeated
    }

    fn mut_field<'a>(&self, m: &'a mut M) -> &'a mut ReflectRepeated {
        (self.mut_vec)(m) as &mut ReflectRepeated
    }
}


pub fn make_repeated_field_accessor<M, V>(
    name: &'static str,
    get_vec: for<'a> fn(&'a M) -> &'a RepeatedField<V::Value>,
    mut_vec: for<'a> fn(&'a mut M) -> &'a mut RepeatedField<V::Value>)
        -> Box<FieldAccessor + 'static>
    where
        M : Message + 'static,
        V : ProtobufType + 'static,
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::RepeatedField(Box::new(RepeatedFieldAccessor::<M, V> {
            get_vec: get_vec,
            mut_vec: mut_vec,
        })),
    })
}



struct MapAccessor<M, K, V>
    where
        M : Message + 'static,
        K : ProtobufType,
        V : ProtobufType,
        <K as ProtobufType>::Value : Hash + Eq,
{
    get_map: for<'a> fn(&'a M) -> &'a HashMap<K::Value, V::Value>,
    mut_map: for<'a> fn(&'a mut M) -> &'a mut HashMap<K::Value, V::Value>,
}

impl<M, K, V> FieldAccessor2<M, ReflectMap> for MapAccessor<M, K, V>
    where
        M : Message + 'static,
        K : ProtobufType,
        V : ProtobufType,
        <K as ProtobufType>::Value : Hash + Eq,
{
    fn get_field<'a>(&self, m: &'a M) -> &'a ReflectMap {
        (self.get_map)(m) as &ReflectMap
    }

    fn mut_field<'a>(&self, m: &'a mut M) -> &'a mut ReflectMap {
        (self.mut_map)(m) as &mut ReflectMap
    }
}

pub fn make_map_accessor<M, K, V>(
    name: &'static str,
    get_map: for<'a> fn(&'a M) -> &'a HashMap<K::Value, V::Value>,
    mut_map: for<'a> fn(&'a mut M) -> &'a mut HashMap<K::Value, V::Value>)
        -> Box<FieldAccessor + 'static>
where
    M : Message + 'static,
    K : ProtobufType + 'static,
    V : ProtobufType + 'static,
    <K as ProtobufType>::Value : Hash + Eq,
{
    Box::new(FieldAccessorImpl {
        name: name,
        fns: FieldAccessorFunctions::Map(Box::new(MapAccessor::<M, K, V> {
            get_map: get_map,
            mut_map: mut_map,
        })),
    })
}
