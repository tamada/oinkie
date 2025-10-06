; ModuleID = 'fizzbuzz.8d6982896d9c1144-cgu.0'
source_filename = "fizzbuzz.8d6982896d9c1144-cgu.0"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "arm64-apple-macosx11.0.0"

%"alloc::string::String" = type { %"alloc::vec::Vec<u8>" }
%"alloc::vec::Vec<u8>" = type { %"alloc::raw_vec::RawVec<u8>", i64 }
%"alloc::raw_vec::RawVec<u8>" = type { %"alloc::raw_vec::RawVecInner", %"core::marker::PhantomData<u8>" }
%"alloc::raw_vec::RawVecInner" = type { i64, ptr, %"alloc::alloc::Global" }
%"alloc::alloc::Global" = type {}
%"core::marker::PhantomData<u8>" = type {}
%"std::ffi::os_str::OsString" = type { %"std::sys::os_str::bytes::Buf" }
%"std::sys::os_str::bytes::Buf" = type { %"alloc::vec::Vec<u8>" }
%"core::fmt::rt::Argument<'_>" = type { %"core::fmt::rt::ArgumentType<'_>" }
%"core::fmt::rt::ArgumentType<'_>" = type { ptr, [1 x i64] }

@vtable.0 = private unnamed_addr constant <{ [24 x i8], ptr, ptr, ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17he1e907c92cf635a5E", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h1137d684aa07bdcfE", ptr @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h1137d684aa07bdcfE" }>, align 8
@vtable.1 = private unnamed_addr constant <{ ptr, [16 x i8], ptr, ptr, ptr }> <{ ptr @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE", [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", ptr @"_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$9write_str17h8c0356c36a6abeafE", ptr @"_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$10write_char17h88aeed3332d36d8bE", ptr @_ZN4core3fmt5Write9write_fmt17h64349c8c9a631612E }>, align 8
@alloc_cc656815297f75969399c3f4b1ad3de4 = private unnamed_addr constant [55 x i8] c"a Display implementation returned an error unexpectedly", align 1
@alloc_1dbd1a3d03a83d9632ea15a97cef1208 = private unnamed_addr constant [109 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/string.rs", align 1
@alloc_9099bd028d7d943c6fbbaae60b97ca36 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1dbd1a3d03a83d9632ea15a97cef1208, [16 x i8] c"m\00\00\00\00\00\00\00\F0\0A\00\00\0E\00\00\00" }>, align 8
@alloc_fad0cd83b7d1858a846a172eb260e593 = private unnamed_addr constant [42 x i8] c"is_aligned_to: align is not a power-of-two", align 1
@alloc_e92e94d0ff530782b571cfd99ec66aef = private unnamed_addr constant <{ ptr, [8 x i8] }> <{ ptr @alloc_fad0cd83b7d1858a846a172eb260e593, [8 x i8] c"*\00\00\00\00\00\00\00" }>, align 8
@anon.2b9cc1fd271beea6e1454a1c8ec7217d.0 = private unnamed_addr constant <{ [8 x i8], [8 x i8] }> <{ [8 x i8] zeroinitializer, [8 x i8] undef }>, align 8
@alloc_ddaf457137c8b66823352a99faaf46ae = private unnamed_addr constant [115 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs", align 1
@alloc_8c596b95dca80098cb9680607f401da7 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_ddaf457137c8b66823352a99faaf46ae, [16 x i8] c"s\00\00\00\00\00\00\00\C4\05\00\00\0D\00\00\00" }>, align 8
@alloc_bd3468a7b96187f70c1ce98a3e7a63bf = private unnamed_addr constant [283 x i8] c"unsafe precondition(s) violated: ptr::copy_nonoverlapping requires that both pointer arguments are aligned and non-null and the specified memory ranges do not overlap\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_a6a0cc8156fe455996de64a9d05b1dfe = private unnamed_addr constant [184 x i8] c"unsafe precondition(s) violated: u32::unchecked_add cannot overflow\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@anon.2b9cc1fd271beea6e1454a1c8ec7217d.1 = private unnamed_addr constant <{ [4 x i8], [4 x i8] }> <{ [4 x i8] zeroinitializer, [4 x i8] undef }>, align 4
@alloc_a19a9eef3136550367377751c21c5637 = private unnamed_addr constant [109 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/num/mod.rs", align 1
@alloc_31affb2d12edf97b24390527480bd5ab = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_a19a9eef3136550367377751c21c5637, [16 x i8] c"m\00\00\00\00\00\00\00N\06\00\00\01\00\00\00" }>, align 8
@alloc_ec595fc0e82ef92fc59bd74f68296eae = private unnamed_addr constant [73 x i8] c"assertion failed: 0 < pointee_size && pointee_size <= isize::MAX as usize", align 1
@alloc_95ce0164b91fa975d5074664dce8a3bd = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_ddaf457137c8b66823352a99faaf46ae, [16 x i8] c"s\00\00\00\00\00\00\00\1E\03\00\00\09\00\00\00" }>, align 8
@alloc_de4e626d456b04760e72bc785ed7e52a = private unnamed_addr constant [201 x i8] c"unsafe precondition(s) violated: ptr::offset_from_unsigned requires `self >= origin`\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_78b6a49e36fe808e0ef950c2feb9fafc = private unnamed_addr constant [71 x i8] c"to_digit: invalid radix -- radix must be in the range 2 to 36 inclusive", align 1
@alloc_708eb9f2492b697e0d761b647be5d46c = private unnamed_addr constant <{ ptr, [8 x i8] }> <{ ptr @alloc_78b6a49e36fe808e0ef950c2feb9fafc, [8 x i8] c"G\00\00\00\00\00\00\00" }>, align 8
@alloc_2f41b8cb76166e311405a5ed6704c83a = private unnamed_addr constant [114 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/char/methods.rs", align 1
@alloc_26977bacf3c1c6974c5960438930cb96 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_2f41b8cb76166e311405a5ed6704c83a, [16 x i8] c"r\00\00\00\00\00\00\00\91\01\00\00\09\00\00\00" }>, align 8
@alloc_64e308ef4babfeb8b6220184de794a17 = private unnamed_addr constant [221 x i8] c"unsafe precondition(s) violated: hint::assert_unchecked must never be called when the condition is false\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@alloc_f0d0a9a84361bc58839021c637852824 = private unnamed_addr constant [124 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/traits/exact_size.rs", align 1
@alloc_2828be464cbb2e0dbc03a0916e604f3b = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_f0d0a9a84361bc58839021c637852824, [16 x i8] c"|\00\00\00\00\00\00\00z\00\00\00\09\00\00\00" }>, align 8
@alloc_fa5c69670c36185e6893e918ff1a021b = private unnamed_addr constant [122 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs", align 1
@alloc_12d3869f6c326732c79369b6c76af0ca = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_fa5c69670c36185e6893e918ff1a021b, [16 x i8] c"z\00\00\00\00\00\00\00\D1\07\00\00\09\00\00\00" }>, align 8
@alloc_b7a8a4de8738df8fbad27c02f1d7f26c = private unnamed_addr constant [111 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/ub_checks.rs", align 1
@alloc_7f6e074e6d0c411207076874a56d8dfb = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_b7a8a4de8738df8fbad27c02f1d7f26c, [16 x i8] c"o\00\00\00\00\00\00\00\86\00\00\006\00\00\00" }>, align 8
@alloc_a28e8c8fd5088943a8b5d44af697ff83 = private unnamed_addr constant [279 x i8] c"unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`\0A\0AThis indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.", align 1
@vtable.2 = private unnamed_addr constant <{ [24 x i8], ptr }> <{ [24 x i8] c"\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00", ptr @"_ZN53_$LT$core..fmt..Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h705ed12b580d2973E" }>, align 8
@alloc_763310d78c99c2c1ad3f8a9821e942f3 = private unnamed_addr constant [61 x i8] c"is_nonoverlapping: `size_of::<T>() * count` overflows a usize", align 1
@alloc_99ac8a81a24cac863217ce4a5cbfabea = private unnamed_addr constant [5 x i8] c"Error", align 1
@alloc_849630626503f5ef0f8eaa4fe7c4e339 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1dbd1a3d03a83d9632ea15a97cef1208, [16 x i8] c"m\00\00\00\00\00\00\00\BB\04\00\00\12\00\00\00" }>, align 8
@alloc_8d1458ac186f4f660817c6f55603b839 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_1dbd1a3d03a83d9632ea15a97cef1208, [16 x i8] c"m\00\00\00\00\00\00\00X\04\00\00\12\00\00\00" }>, align 8
@alloc_f5d936923e395c91d13859e432f0fa23 = private unnamed_addr constant [108 x i8] c"/Users/tamada/.rustup/toolchains/1.88.0-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/slice.rs", align 1
@alloc_2fd90c3491686217d5b4a4abc34a4b75 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_f5d936923e395c91d13859e432f0fa23, [16 x i8] c"l\00\00\00\00\00\00\00\BE\01\00\00\1D\00\00\00" }>, align 8
@alloc_dc62ee45e1b479cd88bad4799c0ef0c2 = private unnamed_addr constant [11 x i8] c"fizzbuzz.rs", align 1
@alloc_2a51b2c50c0c026e4293813ef00e8a1e = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_dc62ee45e1b479cd88bad4799c0ef0c2, [16 x i8] c"\0B\00\00\00\00\00\00\00\05\00\00\00\0D\00\00\00" }>, align 8
@alloc_39d9c0f14cf82e0fad647f0c4a7b4b43 = private unnamed_addr constant [8 x i8] c"FizzBuzz", align 1
@alloc_80a4a3fda200f245ed4df58d4efe532b = private unnamed_addr constant [4 x i8] c"Fizz", align 1
@alloc_2de8df279e307344e74413fa3f65e71d = private unnamed_addr constant [4 x i8] c"Buzz", align 1
@alloc_94b00be069aafad82a2c6df764237b82 = private unnamed_addr constant [2 x i8] c", ", align 1
@alloc_49a1e817e911805af64bbc7efb390101 = private unnamed_addr constant [1 x i8] c"\0A", align 1
@alloc_03a2d32c075c30b45b5a6f468665331f = private unnamed_addr constant <{ ptr, [8 x i8], ptr, [8 x i8], ptr, [8 x i8] }> <{ ptr inttoptr (i64 1 to ptr), [8 x i8] zeroinitializer, ptr @alloc_94b00be069aafad82a2c6df764237b82, [8 x i8] c"\02\00\00\00\00\00\00\00", ptr @alloc_49a1e817e911805af64bbc7efb390101, [8 x i8] c"\01\00\00\00\00\00\00\00" }>, align 8

; <alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::size_hint
; Function Attrs: inlinehint uwtable
define internal void @"_ZN103_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h98898cb255338797E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 {
start:
  %_13 = alloca [16 x i8], align 8
  %exact = alloca [8 x i8], align 8
  br label %bb2

bb2:                                              ; preds = %start
  %_10 = getelementptr inbounds i8, ptr %self, i64 24
  %_8 = load ptr, ptr %_10, align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_11 = load ptr, ptr %0, align 8
; call core::ptr::non_null::NonNull<T>::offset_from_unsigned
  %1 = call i64 @"_ZN4core3ptr8non_null16NonNull$LT$T$GT$20offset_from_unsigned17h914f2be8eb5f0a0dE"(ptr %_8, ptr %_11)
  store i64 %1, ptr %exact, align 8
  br label %bb4

bb4:                                              ; preds = %bb2
  %_12 = load i64, ptr %exact, align 8
  %_14 = load i64, ptr %exact, align 8
  %2 = getelementptr inbounds i8, ptr %_13, i64 8
  store i64 %_14, ptr %2, align 8
  store i64 1, ptr %_13, align 8
  store i64 %_12, ptr %_0, align 8
  %3 = load i64, ptr %_13, align 8
  %4 = getelementptr inbounds i8, ptr %_13, i64 8
  %5 = load i64, ptr %4, align 8
  %6 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 %3, ptr %6, align 8
  %7 = getelementptr inbounds i8, ptr %6, i64 8
  store i64 %5, ptr %7, align 8
  ret void

bb1:                                              ; No predecessors!
  unreachable
}

; <core::ops::range::RangeInclusive<T> as core::iter::range::RangeInclusiveIteratorImpl>::spec_next
; Function Attrs: inlinehint uwtable
define internal { i32, i32 } @"_ZN107_$LT$core..ops..range..RangeInclusive$LT$T$GT$$u20$as$u20$core..iter..range..RangeInclusiveIteratorImpl$GT$9spec_next17h41d95db58e8bf8beE"(ptr align 4 %self) unnamed_addr #0 {
start:
  %_6 = alloca [4 x i8], align 4
  %_0 = alloca [8 x i8], align 4
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %1 = load i8, ptr %0, align 4
  %_10 = trunc nuw i8 %1 to i1
  br i1 %_10, label %bb9, label %bb10

bb10:                                             ; preds = %start
  %_13 = getelementptr inbounds i8, ptr %self, i64 4
  %_3.i = load i32, ptr %self, align 4
  %_4.i = load i32, ptr %_13, align 4
  %_0.i = icmp ule i32 %_3.i, %_4.i
  %_2 = xor i1 %_0.i, true
  br i1 %_2, label %bb1, label %bb2

bb9:                                              ; preds = %start
  br label %bb1

bb2:                                              ; preds = %bb10
  %_5 = getelementptr inbounds i8, ptr %self, i64 4
  %_3.i1 = load i32, ptr %self, align 4
  %_4.i2 = load i32, ptr %_5, align 4
  %_0.i3 = icmp ult i32 %_3.i1, %_4.i2
  br i1 %_0.i3, label %bb4, label %bb6

bb1:                                              ; preds = %bb9, %bb10
  store i32 0, ptr %_0, align 4
  br label %bb8

bb6:                                              ; preds = %bb2
  %2 = getelementptr inbounds i8, ptr %self, i64 8
  store i8 1, ptr %2, align 4
  %3 = load i32, ptr %self, align 4
  store i32 %3, ptr %_6, align 4
  br label %bb7

bb4:                                              ; preds = %bb2
  %_8 = load i32, ptr %self, align 4
; call <u32 as core::iter::range::Step>::forward_unchecked
  %n = call i32 @"_ZN47_$LT$u32$u20$as$u20$core..iter..range..Step$GT$17forward_unchecked17hf01d4f31660e994dE"(i32 %_8, i64 1)
  %4 = load i32, ptr %self, align 4
  store i32 %4, ptr %_6, align 4
  store i32 %n, ptr %self, align 4
  br label %bb7

bb7:                                              ; preds = %bb4, %bb6
  %5 = load i32, ptr %_6, align 4
  %6 = getelementptr inbounds i8, ptr %_0, i64 4
  store i32 %5, ptr %6, align 4
  store i32 1, ptr %_0, align 4
  br label %bb8

bb8:                                              ; preds = %bb1, %bb7
  %7 = load i32, ptr %_0, align 4
  %8 = getelementptr inbounds i8, ptr %_0, i64 4
  %9 = load i32, ptr %8, align 4
  %10 = insertvalue { i32, i32 } poison, i32 %7, 0
  %11 = insertvalue { i32, i32 } %10, i32 %9, 1
  ret { i32, i32 } %11
}

; <alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter
; Function Attrs: uwtable
define internal void @"_ZN111_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter_nested..SpecFromIterNested$LT$T$C$I$GT$$GT$9from_iter17hd0beb9c273219349E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iterator, ptr align 8 %0) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %1 = alloca [8 x i8], align 8
  %2 = alloca [16 x i8], align 8
  %_20 = alloca [1 x i8], align 1
  %_19 = alloca [32 x i8], align 8
  %src = alloca [24 x i8], align 8
  %vector1 = alloca [24 x i8], align 8
  %_8 = alloca [24 x i8], align 8
  %element = alloca [24 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  %vector = alloca [24 x i8], align 8
  store i8 1, ptr %_20, align 1
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::next
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hd466f957c1b1bbd6E"(ptr sret([24 x i8]) align 8 %_3, ptr align 8 %iterator)
          to label %bb1 unwind label %cleanup

bb11:                                             ; preds = %bb9, %bb7, %cleanup
  %3 = load i8, ptr %_20, align 1
  %4 = trunc nuw i8 %3 to i1
  br i1 %4, label %bb10, label %bb8

cleanup:                                          ; preds = %start
  %5 = landingpad { ptr, i32 }
          cleanup
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
  store ptr %6, ptr %2, align 8
  %8 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %7, ptr %8, align 8
  br label %bb11

bb1:                                              ; preds = %start
  %9 = load i64, ptr %_3, align 8
  %10 = icmp eq i64 %9, -9223372036854775808
  %_5 = select i1 %10, i64 0, i64 1
  %11 = trunc nuw i64 %_5 to i1
  br i1 %11, label %bb3, label %bb12

bb3:                                              ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %element, ptr align 8 %_3, i64 24, i1 false)
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::size_hint
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h60b4dc5d4e8c8713E"(ptr sret([24 x i8]) align 8 %_8, ptr align 8 %iterator)
          to label %bb4 unwind label %cleanup2

bb12:                                             ; preds = %bb1
  store i64 0, ptr %_0, align 8
  %12 = getelementptr inbounds i8, ptr %_0, i64 8
  store ptr getelementptr (i8, ptr null, i64 8), ptr %12, align 8
  %13 = getelementptr inbounds i8, ptr %_0, i64 16
  store i64 0, ptr %13, align 8
; call core::ptr::drop_in_place<std::env::Args>
  call void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hf4415d48177fd3c1E"(ptr align 8 %iterator)
  br label %bb6

bb6:                                              ; preds = %bb5, %bb12
  ret void

bb9:                                              ; preds = %cleanup2
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %element) #16
          to label %bb11 unwind label %terminate

cleanup2:                                         ; preds = %bb14, %bb4, %bb3
  %14 = landingpad { ptr, i32 }
          cleanup
  %15 = extractvalue { ptr, i32 } %14, 0
  %16 = extractvalue { ptr, i32 } %14, 1
  store ptr %15, ptr %2, align 8
  %17 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %16, ptr %17, align 8
  br label %bb9

bb4:                                              ; preds = %bb3
  %lower = load i64, ptr %_8, align 8
  %18 = call i64 @llvm.uadd.sat.i64(i64 %lower, i64 1)
  store i64 %18, ptr %1, align 8
  %v2 = load i64, ptr %1, align 8
; invoke core::cmp::Ord::max
  %initial_capacity = invoke i64 @_ZN4core3cmp3Ord3max17h2d6d5446c7ff6005E(i64 4, i64 %v2)
          to label %bb14 unwind label %cleanup2

bb14:                                             ; preds = %bb4
; invoke alloc::raw_vec::RawVecInner<A>::with_capacity_in
  %19 = invoke { i64, ptr } @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$16with_capacity_in17hacdb32290e29d2efE"(i64 %initial_capacity, i64 8, i64 24, ptr align 8 %0)
          to label %bb15 unwind label %cleanup2

bb15:                                             ; preds = %bb14
  %_28.0 = extractvalue { i64, ptr } %19, 0
  %_28.1 = extractvalue { i64, ptr } %19, 1
  store i64 %_28.0, ptr %vector1, align 8
  %20 = getelementptr inbounds i8, ptr %vector1, i64 8
  store ptr %_28.1, ptr %20, align 8
  %21 = getelementptr inbounds i8, ptr %vector1, i64 16
  store i64 0, ptr %21, align 8
  %22 = getelementptr inbounds i8, ptr %vector1, i64 8
  %_29 = load ptr, ptr %22, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %src, ptr align 8 %element, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_29, ptr align 8 %src, i64 24, i1 false)
  %23 = getelementptr inbounds i8, ptr %vector1, i64 16
  store i64 1, ptr %23, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %vector, ptr align 8 %vector1, i64 24, i1 false)
  store i8 0, ptr %_20, align 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_19, ptr align 8 %iterator, i64 32, i1 false)
; invoke <alloc::vec::Vec<T,A> as alloc::vec::spec_extend::SpecExtend<T,I>>::spec_extend
  invoke void @"_ZN97_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$T$C$I$GT$$GT$11spec_extend17h0d29aef41ddbf0c6E"(ptr align 8 %vector, ptr align 8 %_19, ptr align 8 %0)
          to label %bb5 unwind label %cleanup3

bb7:                                              ; preds = %cleanup3
; invoke core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
  invoke void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17h49baea2715f090f9E"(ptr align 8 %vector) #16
          to label %bb11 unwind label %terminate

cleanup3:                                         ; preds = %bb15
  %24 = landingpad { ptr, i32 }
          cleanup
  %25 = extractvalue { ptr, i32 } %24, 0
  %26 = extractvalue { ptr, i32 } %24, 1
  store ptr %25, ptr %2, align 8
  %27 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %26, ptr %27, align 8
  br label %bb7

bb5:                                              ; preds = %bb15
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %vector, i64 24, i1 false)
  br label %bb6

terminate:                                        ; preds = %bb10, %bb9, %bb7
  %28 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %29 = extractvalue { ptr, i32 } %28, 0
  %30 = extractvalue { ptr, i32 } %28, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb2:                                              ; No predecessors!
  unreachable

bb8:                                              ; preds = %bb10, %bb11
  %31 = load ptr, ptr %2, align 8
  %32 = getelementptr inbounds i8, ptr %2, i64 8
  %33 = load i32, ptr %32, align 8
  %34 = insertvalue { ptr, i32 } poison, ptr %31, 0
  %35 = insertvalue { ptr, i32 } %34, i32 %33, 1
  resume { ptr, i32 } %35

bb10:                                             ; preds = %bb11
; invoke core::ptr::drop_in_place<std::env::Args>
  invoke void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hf4415d48177fd3c1E"(ptr align 8 %iterator) #16
          to label %bb8 unwind label %terminate
}

; <<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN157_$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha3ff3f86a3efd8e0E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %capacity = alloca [8 x i8], align 8
  %_4 = alloca [16 x i8], align 8
  %_7 = load ptr, ptr %self, align 8
  %slot = getelementptr inbounds i8, ptr %_7, i64 32
  %_8 = load ptr, ptr %self, align 8
  %ptr = load ptr, ptr %_8, align 8
  %_9 = load ptr, ptr %self, align 8
  %0 = getelementptr inbounds i8, ptr %_9, i64 16
  %capacity1 = load i64, ptr %0, align 8
  br label %bb4

bb4:                                              ; preds = %start
  store i64 %capacity1, ptr %capacity, align 8
  br label %bb2

bb2:                                              ; preds = %bb4
  %cap = load i64, ptr %capacity, align 8
  store i64 %cap, ptr %_4, align 8
  %1 = getelementptr inbounds i8, ptr %_4, i64 8
  store ptr %ptr, ptr %1, align 8
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<std::ffi::os_str::OsString>>
  call void @"_ZN4core3ptr77drop_in_place$LT$alloc..raw_vec..RawVec$LT$std..ffi..os_str..OsString$GT$$GT$17h202b337d44986271E"(ptr align 8 %_4)
  ret void

bb3:                                              ; No predecessors!
  unreachable
}

; std::rt::lang_start
; Function Attrs: uwtable
define hidden i64 @_ZN3std2rt10lang_start17h46345d1e0ea817edE(ptr %main, i64 %argc, ptr %argv, i8 %sigpipe) unnamed_addr #1 {
start:
  %_7 = alloca [8 x i8], align 8
  store ptr %main, ptr %_7, align 8
; call std::rt::lang_start_internal
  %_0 = call i64 @_ZN3std2rt19lang_start_internal17ha7d36e169a6a9a91E(ptr align 1 %_7, ptr align 8 @vtable.0, i64 %argc, ptr %argv, i8 %sigpipe)
  ret i64 %_0
}

; std::rt::lang_start::{{closure}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h1137d684aa07bdcfE"(ptr align 8 %_1) unnamed_addr #0 {
start:
  %_4 = load ptr, ptr %_1, align 8
; call std::sys::backtrace::__rust_begin_short_backtrace
  call void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h25f212add70d90efE(ptr %_4)
; call <() as std::process::Termination>::report
  %self = call i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17h8c9a1574d0d156d5E"()
  %_0 = zext i8 %self to i32
  ret i32 %_0
}

; std::sys::backtrace::__rust_begin_short_backtrace
; Function Attrs: noinline uwtable
define internal void @_ZN3std3sys9backtrace28__rust_begin_short_backtrace17h25f212add70d90efE(ptr %f) unnamed_addr #2 {
start:
; call core::ops::function::FnOnce::call_once
  call void @_ZN4core3ops8function6FnOnce9call_once17h2e8090fc45a939bfE(ptr %f)
  call void asm sideeffect "", "~{memory}"(), !srcloc !3
  ret void
}

; <T as alloc::string::ToString>::to_string
; Function Attrs: inlinehint uwtable
define internal void @"_ZN45_$LT$T$u20$as$u20$alloc..string..ToString$GT$9to_string17h21833b828628f54dE"(ptr sret([24 x i8]) align 8 %_0, ptr align 4 %self) unnamed_addr #0 {
start:
; call <T as alloc::string::SpecToString>::spec_to_string
  call void @"_ZN49_$LT$T$u20$as$u20$alloc..string..SpecToString$GT$14spec_to_string17h0fbada9136ec4277E"(ptr sret([24 x i8]) align 8 %_0, ptr align 4 %self)
  ret void
}

; <T as alloc::string::ToString>::to_string
; Function Attrs: inlinehint uwtable
define internal void @"_ZN45_$LT$T$u20$as$u20$alloc..string..ToString$GT$9to_string17ha7c54548f8a658e9E"(ptr sret([24 x i8]) align 8 %_0, ptr align 1 %self.0, i64 %self.1) unnamed_addr #0 {
start:
; call <str as alloc::string::SpecToString>::spec_to_string
  call void @"_ZN51_$LT$str$u20$as$u20$alloc..string..SpecToString$GT$14spec_to_string17h14e2aefcfd128bddE"(ptr sret([24 x i8]) align 8 %_0, ptr align 1 %self.0, i64 %self.1)
  ret void
}

; <u32 as core::iter::range::Step>::forward_unchecked
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN47_$LT$u32$u20$as$u20$core..iter..range..Step$GT$17forward_unchecked17hf01d4f31660e994dE"(i32 %start1, i64 %n) unnamed_addr #0 {
start:
  %rhs = trunc i64 %n to i32
  br label %bb1

bb1:                                              ; preds = %start
; call core::num::<impl u32>::unchecked_add::precondition_check
  call void @"_ZN4core3num21_$LT$impl$u20$u32$GT$13unchecked_add18precondition_check17hdf2f9b0ac1954d2aE"(i32 %start1, i32 %rhs) #18
  br label %bb2

bb2:                                              ; preds = %bb1
  %_0 = add nuw i32 %start1, %rhs
  ret i32 %_0
}

; <T as alloc::string::SpecToString>::spec_to_string
; Function Attrs: inlinehint uwtable
define internal void @"_ZN49_$LT$T$u20$as$u20$alloc..string..SpecToString$GT$14spec_to_string17h0fbada9136ec4277E"(ptr sret([24 x i8]) align 8 %_0, ptr align 4 %self) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_10 = alloca [24 x i8], align 8
  %options = alloca [8 x i8], align 4
  %formatter = alloca [24 x i8], align 8
  %buf = alloca [24 x i8], align 8
  store i64 0, ptr %_10, align 8
  %1 = getelementptr inbounds i8, ptr %_10, i64 8
  store ptr inttoptr (i64 1 to ptr), ptr %1, align 8
  %2 = getelementptr inbounds i8, ptr %_10, i64 16
  store i64 0, ptr %2, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %buf, ptr align 8 %_10, i64 24, i1 false)
  store i32 -536870880, ptr %options, align 4
  %3 = getelementptr inbounds i8, ptr %options, i64 4
  store i16 0, ptr %3, align 4
  %4 = getelementptr inbounds i8, ptr %options, i64 6
  store i16 0, ptr %4, align 2
  %5 = getelementptr inbounds i8, ptr %formatter, i64 16
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %5, ptr align 4 %options, i64 8, i1 false)
  store ptr %buf, ptr %formatter, align 8
  %6 = getelementptr inbounds i8, ptr %formatter, i64 8
  store ptr @vtable.1, ptr %6, align 8
; invoke core::fmt::num::imp::<impl core::fmt::Display for u32>::fmt
  %_8 = invoke zeroext i1 @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17h34a78d7dcfa5d09eE"(ptr align 4 %self, ptr align 8 %formatter)
          to label %bb1 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %buf) #16
          to label %bb4 unwind label %terminate

cleanup:                                          ; preds = %bb1, %start
  %7 = landingpad { ptr, i32 }
          cleanup
  %8 = extractvalue { ptr, i32 } %7, 0
  %9 = extractvalue { ptr, i32 } %7, 1
  store ptr %8, ptr %0, align 8
  %10 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %9, ptr %10, align 8
  br label %bb3

bb1:                                              ; preds = %start
; invoke core::result::Result<T,E>::expect
  invoke void @"_ZN4core6result19Result$LT$T$C$E$GT$6expect17hc81e96f9cf5b20eaE"(i1 zeroext %_8, ptr align 1 @alloc_cc656815297f75969399c3f4b1ad3de4, i64 55, ptr align 8 @alloc_9099bd028d7d943c6fbbaae60b97ca36)
          to label %bb2 unwind label %cleanup

bb2:                                              ; preds = %bb1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %buf, i64 24, i1 false)
  ret void

terminate:                                        ; preds = %bb3
  %11 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %12 = extractvalue { ptr, i32 } %11, 0
  %13 = extractvalue { ptr, i32 } %11, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb4:                                              ; preds = %bb3
  %14 = load ptr, ptr %0, align 8
  %15 = getelementptr inbounds i8, ptr %0, i64 8
  %16 = load i32, ptr %15, align 8
  %17 = insertvalue { ptr, i32 } poison, ptr %14, 0
  %18 = insertvalue { ptr, i32 } %17, i32 %16, 1
  resume { ptr, i32 } %18
}

; core::intrinsics::copy_nonoverlapping::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core10intrinsics19copy_nonoverlapping18precondition_check17h3b370664dbbe11e2E(ptr %src, ptr %dst, i64 %size, i64 %align, i64 %count) unnamed_addr #3 personality ptr @rust_eh_personality {
start:
  %0 = alloca [4 x i8], align 4
  %_26 = alloca [48 x i8], align 8
  %_21 = alloca [4 x i8], align 4
  %_20 = alloca [8 x i8], align 8
  %_19 = alloca [8 x i8], align 8
  %_18 = alloca [8 x i8], align 8
  %_17 = alloca [48 x i8], align 8
  %is_zst = alloca [1 x i8], align 1
  %align1 = alloca [8 x i8], align 8
  %zero_size = alloca [1 x i8], align 1
  %1 = icmp eq i64 %count, 0
  br i1 %1, label %bb1, label %bb2

bb1:                                              ; preds = %start
  store i8 1, ptr %zero_size, align 1
  store i64 %align, ptr %align1, align 8
  %2 = load i8, ptr %zero_size, align 1
  %3 = trunc nuw i8 %2 to i1
  %4 = zext i1 %3 to i8
  store i8 %4, ptr %is_zst, align 1
  %5 = call i64 @llvm.ctpop.i64(i64 %align)
  %6 = trunc i64 %5 to i32
  store i32 %6, ptr %_21, align 4
  %7 = load i32, ptr %_21, align 4
  %8 = icmp eq i32 %7, 1
  br i1 %8, label %bb26, label %bb15

bb2:                                              ; preds = %start
  %9 = icmp eq i64 %size, 0
  %10 = zext i1 %9 to i8
  store i8 %10, ptr %zero_size, align 1
  store i64 %align, ptr %align1, align 8
  %11 = load i8, ptr %zero_size, align 1
  %12 = trunc nuw i8 %11 to i1
  %13 = zext i1 %12 to i8
  store i8 %13, ptr %is_zst, align 1
  %14 = call i64 @llvm.ctpop.i64(i64 %align)
  %15 = trunc i64 %14 to i32
  store i32 %15, ptr %_21, align 4
  %16 = load i32, ptr %_21, align 4
  %17 = icmp eq i32 %16, 1
  br i1 %17, label %bb14, label %bb15

bb26:                                             ; preds = %bb1
  %18 = ptrtoint ptr %src to i64
  store i64 %18, ptr %_19, align 8
  %19 = sub i64 %align, 1
  store i64 %19, ptr %_20, align 8
  %20 = load i64, ptr %_19, align 8
  %21 = load i64, ptr %_20, align 8
  %22 = and i64 %20, %21
  store i64 %22, ptr %_18, align 8
  %23 = load i64, ptr %_18, align 8
  %24 = icmp eq i64 %23, 0
  br i1 %24, label %bb27, label %bb11

bb15:                                             ; preds = %bb2, %bb1
  store ptr @alloc_e92e94d0ff530782b571cfd99ec66aef, ptr %_17, align 8
  %25 = getelementptr inbounds i8, ptr %_17, i64 8
  store i64 1, ptr %25, align 8
  %26 = load ptr, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, align 8
  %27 = load i64, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, i64 8), align 8
  %28 = getelementptr inbounds i8, ptr %_17, i64 32
  store ptr %26, ptr %28, align 8
  %29 = getelementptr inbounds i8, ptr %28, i64 8
  store i64 %27, ptr %29, align 8
  %30 = getelementptr inbounds i8, ptr %_17, i64 16
  store ptr inttoptr (i64 8 to ptr), ptr %30, align 8
  %31 = getelementptr inbounds i8, ptr %30, i64 8
  store i64 0, ptr %31, align 8
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17heec96bfc27e6c546E(ptr align 8 %_17, ptr align 8 @alloc_8c596b95dca80098cb9680607f401da7) #19
          to label %unreachable unwind label %terminate

bb27:                                             ; preds = %bb26
  br label %bb12

bb11:                                             ; preds = %bb14, %bb26
  br label %bb6

bb12:                                             ; preds = %bb10, %bb27
  br label %bb3

bb14:                                             ; preds = %bb2
  %32 = ptrtoint ptr %src to i64
  store i64 %32, ptr %_19, align 8
  %33 = sub i64 %align, 1
  store i64 %33, ptr %_20, align 8
  %34 = load i64, ptr %_19, align 8
  %35 = load i64, ptr %_20, align 8
  %36 = and i64 %34, %35
  store i64 %36, ptr %_18, align 8
  %37 = load i64, ptr %_18, align 8
  %38 = icmp eq i64 %37, 0
  br i1 %38, label %bb10, label %bb11

bb10:                                             ; preds = %bb14
  %39 = load i8, ptr %is_zst, align 1
  %40 = trunc nuw i8 %39 to i1
  br i1 %40, label %bb12, label %bb13

bb13:                                             ; preds = %bb10
  %41 = load i64, ptr %_19, align 8
  %_15 = icmp eq i64 %41, 0
  %_8 = xor i1 %_15, true
  br i1 %_8, label %bb3, label %bb6

bb6:                                              ; preds = %bb11, %bb13
  br label %bb7

bb3:                                              ; preds = %bb12, %bb13
  %42 = load i8, ptr %zero_size, align 1
  %is_zst2 = trunc nuw i8 %42 to i1
  %43 = call i64 @llvm.ctpop.i64(i64 %align)
  %44 = trunc i64 %43 to i32
  store i32 %44, ptr %0, align 4
  %_29 = load i32, ptr %0, align 4
  %45 = icmp eq i32 %_29, 1
  br i1 %45, label %bb21, label %bb22

bb21:                                             ; preds = %bb3
  %_28 = ptrtoint ptr %dst to i64
  %46 = load i64, ptr %_20, align 8
  %_27 = and i64 %_28, %46
  %47 = icmp eq i64 %_27, 0
  br i1 %47, label %bb17, label %bb18

bb22:                                             ; preds = %bb3
  store ptr @alloc_e92e94d0ff530782b571cfd99ec66aef, ptr %_26, align 8
  %48 = getelementptr inbounds i8, ptr %_26, i64 8
  store i64 1, ptr %48, align 8
  %49 = load ptr, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, align 8
  %50 = load i64, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, i64 8), align 8
  %51 = getelementptr inbounds i8, ptr %_26, i64 32
  store ptr %49, ptr %51, align 8
  %52 = getelementptr inbounds i8, ptr %51, i64 8
  store i64 %50, ptr %52, align 8
  %53 = getelementptr inbounds i8, ptr %_26, i64 16
  store ptr inttoptr (i64 8 to ptr), ptr %53, align 8
  %54 = getelementptr inbounds i8, ptr %53, i64 8
  store i64 0, ptr %54, align 8
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17heec96bfc27e6c546E(ptr align 8 %_26, ptr align 8 @alloc_8c596b95dca80098cb9680607f401da7) #19
          to label %unreachable unwind label %terminate

bb17:                                             ; preds = %bb21
  br i1 %is_zst2, label %bb19, label %bb20

bb18:                                             ; preds = %bb21
  br label %bb5

bb20:                                             ; preds = %bb17
  %_24 = icmp eq i64 %_28, 0
  %_11 = xor i1 %_24, true
  br i1 %_11, label %bb4, label %bb5

bb19:                                             ; preds = %bb17
  br label %bb4

bb5:                                              ; preds = %bb18, %bb20
  br label %bb7

bb4:                                              ; preds = %bb19, %bb20
; invoke core::ub_checks::maybe_is_nonoverlapping::runtime
  %_6 = invoke zeroext i1 @_ZN4core9ub_checks23maybe_is_nonoverlapping7runtime17h87f12e8229fe2a2cE(ptr %src, ptr %dst, i64 %size, i64 %count)
          to label %bb24 unwind label %terminate

terminate:                                        ; preds = %bb15, %bb22, %bb4
  %55 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %56 = extractvalue { ptr, i32 } %55, 0
  %57 = extractvalue { ptr, i32 } %55, 1
; call core::panicking::panic_cannot_unwind
  call void @_ZN4core9panicking19panic_cannot_unwind17he04b9d4fe77d66efE() #17
  unreachable

bb24:                                             ; preds = %bb4
  br i1 %_6, label %bb9, label %bb8

bb8:                                              ; preds = %bb7, %bb24
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17hc7163b0cd384d969E(ptr align 1 @alloc_bd3468a7b96187f70c1ce98a3e7a63bf, i64 283) #20
  unreachable

bb9:                                              ; preds = %bb24
  ret void

bb7:                                              ; preds = %bb6, %bb5
  br label %bb8

unreachable:                                      ; preds = %bb15, %bb22
  unreachable
}

; core::intrinsics::is_val_statically_known
; Function Attrs: nounwind uwtable
define internal zeroext i1 @_ZN4core10intrinsics23is_val_statically_known17he60365e7f65146dbE(i1 zeroext %_arg) unnamed_addr #4 {
start:
  ret i1 false
}

; core::intrinsics::cold_path
; Function Attrs: cold nounwind uwtable
define internal void @_ZN4core10intrinsics9cold_path17hbfdde3684970a915E() unnamed_addr #5 {
start:
  ret void
}

; core::cmp::Ord::max
; Function Attrs: inlinehint uwtable
define internal i64 @_ZN4core3cmp3Ord3max17h2d6d5446c7ff6005E(i64 %0, i64 %1) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %2 = alloca [16 x i8], align 8
  %_6 = alloca [1 x i8], align 1
  %_0 = alloca [8 x i8], align 8
  %other = alloca [8 x i8], align 8
  %self = alloca [8 x i8], align 8
  store i64 %0, ptr %self, align 8
  store i64 %1, ptr %other, align 8
  store i8 1, ptr %_6, align 1
  %_3.i = load i64, ptr %other, align 8
  %_4.i = load i64, ptr %self, align 8
  %_0.i = icmp ult i64 %_3.i, %_4.i
  br label %bb1

bb5:                                              ; preds = %cleanup
  br label %bb9

cleanup:                                          ; No predecessors!
  %3 = landingpad { ptr, i32 }
          cleanup
  %4 = extractvalue { ptr, i32 } %3, 0
  %5 = extractvalue { ptr, i32 } %3, 1
  store ptr %4, ptr %2, align 8
  %6 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %5, ptr %6, align 8
  br label %bb5

bb1:                                              ; preds = %start
  br i1 %_0.i, label %bb2, label %bb3

bb3:                                              ; preds = %bb1
  %7 = load i64, ptr %other, align 8
  store i64 %7, ptr %_0, align 8
  %8 = load i8, ptr %_6, align 1
  %9 = trunc nuw i8 %8 to i1
  br i1 %9, label %bb7, label %bb4

bb2:                                              ; preds = %bb1
  store i8 0, ptr %_6, align 1
  %10 = load i64, ptr %self, align 8
  store i64 %10, ptr %_0, align 8
  br label %bb4

bb4:                                              ; preds = %bb2, %bb7, %bb3
  %11 = load i64, ptr %_0, align 8
  ret i64 %11

bb7:                                              ; preds = %bb3
  br label %bb4

bb9:                                              ; preds = %bb5
  %12 = load i8, ptr %_6, align 1
  %13 = trunc nuw i8 %12 to i1
  br i1 %13, label %bb8, label %bb6

bb6:                                              ; preds = %bb8, %bb9
  %14 = load ptr, ptr %2, align 8
  %15 = getelementptr inbounds i8, ptr %2, i64 8
  %16 = load i32, ptr %15, align 8
  %17 = insertvalue { ptr, i32 } poison, ptr %14, 0
  %18 = insertvalue { ptr, i32 } %17, i32 %16, 1
  resume { ptr, i32 } %18

bb8:                                              ; preds = %bb9
  br label %bb6
}

; core::fmt::rt::<impl core::fmt::Arguments>::new_v1
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3fmt2rt38_$LT$impl$u20$core..fmt..Arguments$GT$6new_v117h725926bb9b040646E"(ptr sret([48 x i8]) align 8 %_0, ptr align 8 %pieces, ptr align 8 %args) unnamed_addr #0 {
start:
  store ptr %pieces, ptr %_0, align 8
  %0 = getelementptr inbounds i8, ptr %_0, i64 8
  store i64 3, ptr %0, align 8
  %1 = load ptr, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, align 8
  %2 = load i64, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, i64 8), align 8
  %3 = getelementptr inbounds i8, ptr %_0, i64 32
  store ptr %1, ptr %3, align 8
  %4 = getelementptr inbounds i8, ptr %3, i64 8
  store i64 %2, ptr %4, align 8
  %5 = getelementptr inbounds i8, ptr %_0, i64 16
  store ptr %args, ptr %5, align 8
  %6 = getelementptr inbounds i8, ptr %5, i64 8
  store i64 2, ptr %6, align 8
  ret void
}

; core::fmt::rt::Argument::new_display
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core3fmt2rt8Argument11new_display17h3377736cb327981aE(ptr sret([16 x i8]) align 8 %_0, ptr align 4 %x) unnamed_addr #0 {
start:
  %_2 = alloca [16 x i8], align 8
  store ptr %x, ptr %_2, align 8
  %0 = getelementptr inbounds i8, ptr %_2, i64 8
  store ptr @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17h34a78d7dcfa5d09eE", ptr %0, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %_2, i64 16, i1 false)
  ret void
}

; core::fmt::rt::Argument::new_display
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core3fmt2rt8Argument11new_display17hef888bca4d0b8c30E(ptr sret([16 x i8]) align 8 %_0, ptr align 8 %x) unnamed_addr #0 {
start:
  %_2 = alloca [16 x i8], align 8
  store ptr %x, ptr %_2, align 8
  %0 = getelementptr inbounds i8, ptr %_2, i64 8
  store ptr @"_ZN60_$LT$alloc..string..String$u20$as$u20$core..fmt..Display$GT$3fmt17hbe151b8cb91623eaE", ptr %0, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %_2, i64 16, i1 false)
  ret void
}

; core::fmt::Write::write_fmt
; Function Attrs: uwtable
define internal zeroext i1 @_ZN4core3fmt5Write9write_fmt17h64349c8c9a631612E(ptr align 8 %self, ptr align 8 %args) unnamed_addr #1 {
start:
; call <&mut W as core::fmt::Write::write_fmt::SpecWriteFmt>::spec_write_fmt
  %_0 = call zeroext i1 @"_ZN75_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write..write_fmt..SpecWriteFmt$GT$14spec_write_fmt17hb107362395a091c1E"(ptr align 8 %self, ptr align 8 %args)
  ret i1 %_0
}

; core::num::<impl u32>::unchecked_add::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN4core3num21_$LT$impl$u20$u32$GT$13unchecked_add18precondition_check17hdf2f9b0ac1954d2aE"(i32 %lhs, i32 %rhs) unnamed_addr #3 {
start:
  %0 = call { i32, i1 } @llvm.uadd.with.overflow.i32(i32 %lhs, i32 %rhs)
  %_5.0 = extractvalue { i32, i1 } %0, 0
  %_5.1 = extractvalue { i32, i1 } %0, 1
  br i1 %_5.1, label %bb1, label %bb2

bb2:                                              ; preds = %start
  ret void

bb1:                                              ; preds = %start
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17hc7163b0cd384d969E(ptr align 1 @alloc_a6a0cc8156fe455996de64a9d05b1dfe, i64 184) #20
  unreachable
}

; core::num::<impl u32>::from_ascii_radix
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN4core3num21_$LT$impl$u20$u32$GT$16from_ascii_radix17h60907631930e6b7cE"(ptr align 1 %src.0, i64 %src.1, i32 %radix) unnamed_addr #0 {
start:
  %_76 = alloca [16 x i8], align 8
  %_75 = alloca [1 x i8], align 1
  %_64 = alloca [8 x i8], align 8
  %_63 = alloca [1 x i8], align 1
  %_45 = alloca [8 x i8], align 4
  %value = alloca [4 x i8], align 4
  %_42 = alloca [8 x i8], align 8
  %_41 = alloca [1 x i8], align 1
  %_40 = alloca [4 x i8], align 4
  %_39 = alloca [8 x i8], align 4
  %x = alloca [4 x i8], align 4
  %mul = alloca [8 x i8], align 4
  %_22 = alloca [8 x i8], align 4
  %digits2 = alloca [16 x i8], align 8
  %radix1 = alloca [4 x i8], align 4
  %_14 = alloca [1 x i8], align 1
  %result = alloca [4 x i8], align 4
  %rest = alloca [16 x i8], align 8
  %digits = alloca [16 x i8], align 8
  %is_positive = alloca [1 x i8], align 1
  %_0 = alloca [8 x i8], align 4
  %_3 = icmp ugt i32 2, %radix
  br i1 %_3, label %bb2, label %bb1

bb1:                                              ; preds = %start
  %_4 = icmp ugt i32 %radix, 36
  br i1 %_4, label %bb2, label %bb3

bb2:                                              ; preds = %bb1, %start
; call core::num::from_ascii_radix_panic
  call void @_ZN4core3num22from_ascii_radix_panic17hdce6039c94e6631fE(i32 %radix, ptr align 8 @alloc_31affb2d12edf97b24390527480bd5ab) #19
  unreachable

bb3:                                              ; preds = %bb1
  %0 = icmp eq i64 %src.1, 0
  br i1 %0, label %bb4, label %bb5

bb4:                                              ; preds = %bb3
  %1 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 0, ptr %1, align 1
  store i8 1, ptr %_0, align 4
  br label %bb29

bb5:                                              ; preds = %bb3
  %2 = icmp eq i64 %src.1, 1
  br i1 %2, label %bb7, label %bb6

bb29:                                             ; preds = %bb28, %bb26, %bb12, %bb4
  %3 = load i64, ptr %_0, align 4
  ret i64 %3

bb7:                                              ; preds = %bb5
  %4 = getelementptr inbounds nuw i8, ptr %src.0, i64 0
  %5 = load i8, ptr %4, align 1
  switch i8 %5, label %bb6 [
    i8 43, label %bb12
    i8 45, label %bb12
  ]

bb6:                                              ; preds = %bb7, %bb5
  %_9 = icmp uge i64 %src.1, 1
  br i1 %_9, label %bb9, label %bb8

bb12:                                             ; preds = %bb7, %bb7
  %6 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 1, ptr %6, align 1
  store i8 1, ptr %_0, align 4
  br label %bb29

bb8:                                              ; preds = %bb10, %bb9, %bb6
  store i8 1, ptr %_75, align 1
  store ptr %src.0, ptr %_76, align 8
  %7 = getelementptr inbounds i8, ptr %_76, i64 8
  store i64 %src.1, ptr %7, align 8
  %8 = load i8, ptr %_75, align 1
  %9 = trunc nuw i8 %8 to i1
  %10 = zext i1 %9 to i8
  store i8 %10, ptr %is_positive, align 1
  %11 = load ptr, ptr %_76, align 8
  %12 = getelementptr inbounds i8, ptr %_76, i64 8
  %13 = load i64, ptr %12, align 8
  store ptr %11, ptr %digits, align 8
  %14 = getelementptr inbounds i8, ptr %digits, i64 8
  store i64 %13, ptr %14, align 8
  store i32 0, ptr %result, align 4
  store i32 %radix, ptr %radix1, align 4
  %15 = load ptr, ptr %digits, align 8
  %16 = getelementptr inbounds i8, ptr %digits, i64 8
  %17 = load i64, ptr %16, align 8
  store ptr %15, ptr %digits2, align 8
  %18 = getelementptr inbounds i8, ptr %digits2, i64 8
  store i64 %17, ptr %18, align 8
  %19 = icmp ule i32 %radix, 16
  %20 = zext i1 %19 to i8
  store i8 %20, ptr %_63, align 1
  %21 = load i8, ptr %_63, align 1
  %22 = trunc nuw i8 %21 to i1
  br i1 %22, label %bb37, label %bb41

bb9:                                              ; preds = %bb6
  %23 = getelementptr inbounds nuw i8, ptr %src.0, i64 0
  %24 = load i8, ptr %23, align 1
  switch i8 %24, label %bb8 [
    i8 43, label %bb11
    i8 45, label %bb10
  ]

bb11:                                             ; preds = %bb9
  %rest.0 = getelementptr inbounds nuw i8, ptr %src.0, i64 1
  %rest.1 = sub i64 %src.1, 1
  store i8 1, ptr %_75, align 1
  store ptr %rest.0, ptr %_76, align 8
  %25 = getelementptr inbounds i8, ptr %_76, i64 8
  store i64 %rest.1, ptr %25, align 8
  %26 = load i8, ptr %_75, align 1
  %27 = trunc nuw i8 %26 to i1
  %28 = zext i1 %27 to i8
  store i8 %28, ptr %is_positive, align 1
  %29 = load ptr, ptr %_76, align 8
  %30 = getelementptr inbounds i8, ptr %_76, i64 8
  %31 = load i64, ptr %30, align 8
  store ptr %29, ptr %digits, align 8
  %32 = getelementptr inbounds i8, ptr %digits, i64 8
  store i64 %31, ptr %32, align 8
  store i32 0, ptr %result, align 4
  store i32 %radix, ptr %radix1, align 4
  %33 = load ptr, ptr %digits, align 8
  %34 = getelementptr inbounds i8, ptr %digits, i64 8
  %35 = load i64, ptr %34, align 8
  store ptr %33, ptr %digits2, align 8
  %36 = getelementptr inbounds i8, ptr %digits2, i64 8
  store i64 %35, ptr %36, align 8
  %37 = icmp ule i32 %radix, 16
  %38 = zext i1 %37 to i8
  store i8 %38, ptr %_63, align 1
  %39 = load i8, ptr %_63, align 1
  %40 = trunc nuw i8 %39 to i1
  br i1 %40, label %bb39, label %bb43

bb10:                                             ; preds = %bb9
  %41 = getelementptr inbounds nuw i8, ptr %src.0, i64 1
  %42 = sub i64 %src.1, 1
  store ptr %41, ptr %rest, align 8
  %43 = getelementptr inbounds i8, ptr %rest, i64 8
  store i64 %42, ptr %43, align 8
  br label %bb8

bb43:                                             ; preds = %bb11
  store i8 0, ptr %_14, align 1
  %44 = load i8, ptr %_14, align 1
  %45 = trunc nuw i8 %44 to i1
  br i1 %45, label %bb44, label %bb48

bb39:                                             ; preds = %bb11
  %46 = load ptr, ptr %digits2, align 8
  %47 = getelementptr inbounds i8, ptr %digits2, i64 8
  %48 = load i64, ptr %47, align 8
  store i64 %48, ptr %_64, align 8
  %49 = load i64, ptr %_64, align 8
  %50 = icmp ule i64 %49, 8
  %51 = zext i1 %50 to i8
  store i8 %51, ptr %_14, align 1
  %52 = load i8, ptr %_14, align 1
  %53 = trunc nuw i8 %52 to i1
  br i1 %53, label %bb40, label %bb46

bb48:                                             ; preds = %bb43
  br label %bb19.preheader

bb44:                                             ; preds = %bb43
  unreachable

bb19:                                             ; preds = %bb19.preheader, %bb25
  %54 = load ptr, ptr %digits, align 8
  %55 = getelementptr inbounds i8, ptr %digits, i64 8
  %_32 = load i64, ptr %55, align 8
  %_33 = icmp uge i64 %_32, 1
  br i1 %_33, label %bb20, label %bb26.loopexit12

bb46:                                             ; preds = %bb39
  br label %bb19.preheader

bb40:                                             ; preds = %bb39
  br label %bb13.preheader

bb13.preheader:                                   ; preds = %bb40, %bb38
  br label %bb13

bb13:                                             ; preds = %bb13.preheader, %bb18
  %56 = load ptr, ptr %digits, align 8
  %57 = getelementptr inbounds i8, ptr %digits, i64 8
  %_17 = load i64, ptr %57, align 8
  %_18 = icmp uge i64 %_17, 1
  br i1 %_18, label %bb14, label %bb26.loopexit

bb41:                                             ; preds = %bb8
  store i8 0, ptr %_14, align 1
  %58 = load i8, ptr %_14, align 1
  %59 = trunc nuw i8 %58 to i1
  br i1 %59, label %bb42, label %bb47

bb37:                                             ; preds = %bb8
  %60 = load ptr, ptr %digits2, align 8
  %61 = getelementptr inbounds i8, ptr %digits2, i64 8
  %62 = load i64, ptr %61, align 8
  store i64 %62, ptr %_64, align 8
  %63 = load i64, ptr %_64, align 8
  %64 = icmp ule i64 %63, 8
  %65 = zext i1 %64 to i8
  store i8 %65, ptr %_14, align 1
  %66 = load i8, ptr %_14, align 1
  %67 = trunc nuw i8 %66 to i1
  br i1 %67, label %bb38, label %bb45

bb47:                                             ; preds = %bb41
  br label %bb19.preheader

bb42:                                             ; preds = %bb41
  unreachable

bb45:                                             ; preds = %bb37
  br label %bb19.preheader

bb19.preheader:                                   ; preds = %bb48, %bb46, %bb47, %bb45
  br label %bb19

bb38:                                             ; preds = %bb37
  br label %bb13.preheader

bb26.loopexit:                                    ; preds = %bb13
  br label %bb26

bb26.loopexit12:                                  ; preds = %bb19
  br label %bb26

bb26:                                             ; preds = %bb26.loopexit12, %bb26.loopexit
  %_61 = load i32, ptr %result, align 4
  %68 = getelementptr inbounds i8, ptr %_0, i64 4
  store i32 %_61, ptr %68, align 4
  store i8 0, ptr %_0, align 4
  br label %bb29

bb20:                                             ; preds = %bb19
  %69 = load ptr, ptr %digits, align 8
  %70 = getelementptr inbounds i8, ptr %digits, i64 8
  %71 = load i64, ptr %70, align 8
  %c = getelementptr inbounds nuw i8, ptr %69, i64 0
  %72 = load ptr, ptr %digits, align 8
  %73 = getelementptr inbounds i8, ptr %digits, i64 8
  %74 = load i64, ptr %73, align 8
  %rest.03 = getelementptr inbounds nuw i8, ptr %72, i64 1
  %rest.14 = sub i64 %74, 1
  %self = load i32, ptr %result, align 4
  %75 = call { i32, i1 } @llvm.umul.with.overflow.i32(i32 %self, i32 %radix)
  %_67.0 = extractvalue { i32, i1 } %75, 0
  %_67.1 = extractvalue { i32, i1 } %75, 1
  br i1 %_67.1, label %bb30, label %bb32

bb32:                                             ; preds = %bb20
  %76 = getelementptr inbounds i8, ptr %mul, i64 4
  store i32 %_67.0, ptr %76, align 4
  store i32 1, ptr %mul, align 4
  %77 = load i8, ptr %c, align 1
  store i8 %77, ptr %_41, align 1
  %78 = load i8, ptr %_41, align 1
  %79 = zext i8 %78 to i32
  store i32 %79, ptr %_40, align 4
  %80 = load i32, ptr %_40, align 4
; call core::char::methods::<impl char>::to_digit
  %81 = call { i32, i32 } @"_ZN4core4char7methods22_$LT$impl$u20$char$GT$8to_digit17hd0ac33e4739e6dccE"(i32 %80, i32 %radix)
  %82 = extractvalue { i32, i32 } %81, 0
  %83 = extractvalue { i32, i32 } %81, 1
  store i32 %82, ptr %_39, align 4
  %84 = getelementptr inbounds i8, ptr %_39, i64 4
  store i32 %83, ptr %84, align 4
  %85 = load i32, ptr %_39, align 4
  %86 = getelementptr inbounds i8, ptr %_39, i64 4
  %87 = load i32, ptr %86, align 4
  %88 = zext i32 %85 to i64
  store i64 %88, ptr %_42, align 8
  %89 = load i64, ptr %_42, align 8
  %90 = trunc nuw i64 %89 to i1
  br i1 %90, label %bb23, label %bb22.loopexit

bb30:                                             ; preds = %bb20
  %c.lcssa = phi ptr [ %c, %bb20 ]
  %91 = load i32, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.1, align 4
  %92 = load i32, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.1, i64 4), align 4
  store i32 %91, ptr %mul, align 4
  %93 = getelementptr inbounds i8, ptr %mul, i64 4
  store i32 %92, ptr %93, align 4
  %94 = load i8, ptr %c.lcssa, align 1
  store i8 %94, ptr %_41, align 1
  %95 = load i8, ptr %_41, align 1
  %96 = zext i8 %95 to i32
  store i32 %96, ptr %_40, align 4
  %97 = load i32, ptr %_40, align 4
; call core::char::methods::<impl char>::to_digit
  %98 = call { i32, i32 } @"_ZN4core4char7methods22_$LT$impl$u20$char$GT$8to_digit17hd0ac33e4739e6dccE"(i32 %97, i32 %radix)
  %99 = extractvalue { i32, i32 } %98, 0
  %100 = extractvalue { i32, i32 } %98, 1
  store i32 %99, ptr %_39, align 4
  %101 = getelementptr inbounds i8, ptr %_39, i64 4
  store i32 %100, ptr %101, align 4
  %102 = load i32, ptr %_39, align 4
  %103 = getelementptr inbounds i8, ptr %_39, i64 4
  %104 = load i32, ptr %103, align 4
  %105 = zext i32 %102 to i64
  store i64 %105, ptr %_42, align 8
  %106 = load i64, ptr %_42, align 8
  %107 = trunc nuw i64 %106 to i1
  br i1 %107, label %bb50, label %bb22

bb23:                                             ; preds = %bb32
  %108 = getelementptr inbounds i8, ptr %_39, i64 4
  %109 = load i32, ptr %108, align 4
  store i32 %109, ptr %value, align 4
  %110 = load i32, ptr %value, align 4
  store i32 %110, ptr %x, align 4
  %111 = getelementptr inbounds i8, ptr %mul, i64 4
  %value5 = load i32, ptr %111, align 4
  store i32 %value5, ptr %result, align 4
  %self6 = load i32, ptr %result, align 4
  %rhs = load i32, ptr %value, align 4
  %112 = load i32, ptr %value, align 4
  %113 = call { i32, i1 } @llvm.uadd.with.overflow.i32(i32 %self6, i32 %112)
  %_70.0 = extractvalue { i32, i1 } %113, 0
  %_70.1 = extractvalue { i32, i1 } %113, 1
  br i1 %_70.1, label %bb34, label %bb36

bb22.loopexit:                                    ; preds = %bb32
  br label %bb22

bb22:                                             ; preds = %bb22.loopexit, %bb30
  %114 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 1, ptr %114, align 1
  store i8 1, ptr %_0, align 4
  br label %bb27

bb36:                                             ; preds = %bb23
  %115 = load i32, ptr %value, align 4
  %_71 = add nuw i32 %self6, %115
  %116 = getelementptr inbounds i8, ptr %_45, i64 4
  store i32 %_71, ptr %116, align 4
  store i32 1, ptr %_45, align 4
  br label %bb33

bb34:                                             ; preds = %bb23
  %117 = load i32, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.1, align 4
  %118 = load i32, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.1, i64 4), align 4
  store i32 %117, ptr %_45, align 4
  %119 = getelementptr inbounds i8, ptr %_45, i64 4
  store i32 %118, ptr %119, align 4
  br label %bb33

bb33:                                             ; preds = %bb34, %bb36
  %120 = load i32, ptr %_45, align 4
  %121 = getelementptr inbounds i8, ptr %_45, i64 4
  %122 = load i32, ptr %121, align 4
  %_48 = zext i32 %120 to i64
  %123 = trunc nuw i64 %_48 to i1
  br i1 %123, label %bb25, label %bb24

bb25:                                             ; preds = %bb33
  %124 = getelementptr inbounds i8, ptr %_45, i64 4
  %value7 = load i32, ptr %124, align 4
  store i32 %value7, ptr %result, align 4
  store ptr %rest.03, ptr %digits, align 8
  %125 = getelementptr inbounds i8, ptr %digits, i64 8
  store i64 %rest.14, ptr %125, align 8
  br label %bb19

bb24:                                             ; preds = %bb33
  %126 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 2, ptr %126, align 1
  store i8 1, ptr %_0, align 4
  br label %bb27

bb27:                                             ; preds = %bb50, %bb22, %bb24
  br label %bb28

bb50:                                             ; preds = %bb30
  %127 = getelementptr inbounds i8, ptr %_39, i64 4
  %128 = load i32, ptr %127, align 4
  store i32 %128, ptr %value, align 4
  %129 = load i32, ptr %value, align 4
  store i32 %129, ptr %x, align 4
  %130 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 2, ptr %130, align 1
  store i8 1, ptr %_0, align 4
  br label %bb27

bb28:                                             ; preds = %bb17, %bb27
  br label %bb29

bb14:                                             ; preds = %bb13
  %131 = load ptr, ptr %digits, align 8
  %132 = getelementptr inbounds i8, ptr %digits, i64 8
  %133 = load i64, ptr %132, align 8
  %c8 = getelementptr inbounds nuw i8, ptr %131, i64 0
  %134 = load ptr, ptr %digits, align 8
  %135 = getelementptr inbounds i8, ptr %digits, i64 8
  %136 = load i64, ptr %135, align 8
  %rest.09 = getelementptr inbounds nuw i8, ptr %134, i64 1
  %rest.110 = sub i64 %136, 1
  %_21 = load i32, ptr %result, align 4
  %137 = mul i32 %_21, %radix
  store i32 %137, ptr %result, align 4
  %_24 = load i8, ptr %c8, align 1
  %_23 = zext i8 %_24 to i32
; call core::char::methods::<impl char>::to_digit
  %138 = call { i32, i32 } @"_ZN4core4char7methods22_$LT$impl$u20$char$GT$8to_digit17hd0ac33e4739e6dccE"(i32 %_23, i32 %radix)
  %139 = extractvalue { i32, i32 } %138, 0
  %140 = extractvalue { i32, i32 } %138, 1
  store i32 %139, ptr %_22, align 4
  %141 = getelementptr inbounds i8, ptr %_22, i64 4
  store i32 %140, ptr %141, align 4
  %142 = load i32, ptr %_22, align 4
  %143 = getelementptr inbounds i8, ptr %_22, i64 4
  %144 = load i32, ptr %143, align 4
  %_25 = zext i32 %142 to i64
  %145 = trunc nuw i64 %_25 to i1
  br i1 %145, label %bb18, label %bb17

bb18:                                             ; preds = %bb14
  %146 = getelementptr inbounds i8, ptr %_22, i64 4
  %x11 = load i32, ptr %146, align 4
  %_27 = load i32, ptr %result, align 4
  %147 = add i32 %_27, %x11
  store i32 %147, ptr %result, align 4
  store ptr %rest.09, ptr %digits, align 8
  %148 = getelementptr inbounds i8, ptr %digits, i64 8
  store i64 %rest.110, ptr %148, align 8
  br label %bb13

bb17:                                             ; preds = %bb14
  %149 = getelementptr inbounds i8, ptr %_0, i64 1
  store i8 1, ptr %149, align 1
  store i8 1, ptr %_0, align 4
  br label %bb28

bb16:                                             ; No predecessors!
  unreachable
}

; core::num::<impl core::str::traits::FromStr for u32>::from_str
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN4core3num60_$LT$impl$u20$core..str..traits..FromStr$u20$for$u20$u32$GT$8from_str17h0f2039747053d208E"(ptr align 1 %src.0, i64 %src.1) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %_0 = alloca [8 x i8], align 4
; call core::num::<impl u32>::from_ascii_radix
  %1 = call i64 @"_ZN4core3num21_$LT$impl$u20$u32$GT$16from_ascii_radix17h60907631930e6b7cE"(ptr align 1 %src.0, i64 %src.1, i32 10)
  store i64 %1, ptr %0, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %_0, ptr align 8 %0, i64 8, i1 false)
  %2 = load i64, ptr %_0, align 4
  ret i64 %2
}

; core::ops::range::RangeInclusive<Idx>::new
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ops5range25RangeInclusive$LT$Idx$GT$3new17h21c59fb822df068dE"(ptr sret([12 x i8]) align 4 %_0, i32 %start1, i32 %end) unnamed_addr #0 {
start:
  store i32 %start1, ptr %_0, align 4
  %0 = getelementptr inbounds i8, ptr %_0, i64 4
  store i32 %end, ptr %0, align 4
  %1 = getelementptr inbounds i8, ptr %_0, i64 8
  store i8 0, ptr %1, align 4
  ret void
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17he1e907c92cf635a5E"(ptr %_1) unnamed_addr #0 {
start:
  %_2 = alloca [0 x i8], align 1
  %0 = load ptr, ptr %_1, align 8
; call core::ops::function::FnOnce::call_once
  %_0 = call i32 @_ZN4core3ops8function6FnOnce9call_once17hcb9df34d2c325e3fE(ptr %0)
  ret i32 %_0
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17h2e8090fc45a939bfE(ptr %_1) unnamed_addr #0 {
start:
  %_2 = alloca [0 x i8], align 1
  call void %_1()
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint uwtable
define internal i32 @_ZN4core3ops8function6FnOnce9call_once17hcb9df34d2c325e3fE(ptr %0) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %_2 = alloca [0 x i8], align 1
  %_1 = alloca [8 x i8], align 8
  store ptr %0, ptr %_1, align 8
; invoke std::rt::lang_start::{{closure}}
  %_0 = invoke i32 @"_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17h1137d684aa07bdcfE"(ptr align 8 %_1)
          to label %bb1 unwind label %cleanup

bb3:                                              ; preds = %cleanup
  %2 = load ptr, ptr %1, align 8
  %3 = getelementptr inbounds i8, ptr %1, i64 8
  %4 = load i32, ptr %3, align 8
  %5 = insertvalue { ptr, i32 } poison, ptr %2, 0
  %6 = insertvalue { ptr, i32 } %5, i32 %4, 1
  resume { ptr, i32 } %6

cleanup:                                          ; preds = %start
  %7 = landingpad { ptr, i32 }
          cleanup
  %8 = extractvalue { ptr, i32 } %7, 0
  %9 = extractvalue { ptr, i32 } %7, 1
  store ptr %8, ptr %1, align 8
  %10 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %9, ptr %10, align 8
  br label %bb3

bb1:                                              ; preds = %start
  ret i32 %_0
}

; core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17ha099effc7cff476aE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN157_$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha3ff3f86a3efd8e0E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::env::Args>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hf4415d48177fd3c1E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::env::ArgsOs>
  call void @"_ZN4core3ptr37drop_in_place$LT$std..env..ArgsOs$GT$17h3873d7a8ae5a6654E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<core::fmt::Error>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr37drop_in_place$LT$core..fmt..Error$GT$17h26c459937c59a997E"(ptr align 1 %_1) unnamed_addr #0 {
start:
  ret void
}

; core::ptr::drop_in_place<std::env::ArgsOs>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr37drop_in_place$LT$std..env..ArgsOs$GT$17h3873d7a8ae5a6654E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::sys::args::common::Args>
  call void @"_ZN4core3ptr49drop_in_place$LT$std..sys..args..common..Args$GT$17ha1221e1cd0386c3eE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::string::String>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h7a288ed628364c70E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::vec::Vec<u8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h7a288ed628364c70E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3c84f8633605c81bE"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
  invoke void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17h96750253b9c6fdb1E"(ptr align 8 %_1) #16
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb3

bb4:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
  call void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17h96750253b9c6fdb1E"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb1:                                              ; preds = %bb3
  %8 = load ptr, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  %10 = load i32, ptr %9, align 8
  %11 = insertvalue { ptr, i32 } poison, ptr %8, 0
  %12 = insertvalue { ptr, i32 } %11, i32 %10, 1
  resume { ptr, i32 } %12
}

; core::ptr::drop_in_place<std::ffi::os_str::OsString>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17he8bedefff6f040d8E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<std::sys::os_str::bytes::Buf>
  call void @"_ZN4core3ptr49drop_in_place$LT$std..sys..os_str..bytes..Buf$GT$17h71693174271aa2ebE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::sys::args::common::Args>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr49drop_in_place$LT$std..sys..args..common..Args$GT$17ha1221e1cd0386c3eE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::into_iter::IntoIter<std::ffi::os_str::OsString>>
  call void @"_ZN4core3ptr86drop_in_place$LT$alloc..vec..into_iter..IntoIter$LT$std..ffi..os_str..OsString$GT$$GT$17h7aaa73b9490f4a1bE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::sys::os_str::bytes::Buf>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr49drop_in_place$LT$std..sys..os_str..bytes..Buf$GT$17h71693174271aa2ebE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call core::ptr::drop_in_place<alloc::vec::Vec<u8>>
  call void @"_ZN4core3ptr46drop_in_place$LT$alloc..vec..Vec$LT$u8$GT$$GT$17h7a288ed628364c70E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<[alloc::string::String]>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr52drop_in_place$LT$$u5b$alloc..string..String$u5d$$GT$17hff8f8410fae3b5cdE"(ptr align 8 %_1.0, i64 %_1.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_3 = alloca [8 x i8], align 8
  store i64 0, ptr %_3, align 8
  br label %bb6

bb6:                                              ; preds = %bb5, %start
  %1 = load i64, ptr %_3, align 8
  %_7 = icmp eq i64 %1, %_1.1
  br i1 %_7, label %bb1, label %bb5

bb5:                                              ; preds = %bb6
  %2 = load i64, ptr %_3, align 8
  %_6 = getelementptr inbounds nuw %"alloc::string::String", ptr %_1.0, i64 %2
  %3 = load i64, ptr %_3, align 8
  %4 = add i64 %3, 1
  store i64 %4, ptr %_3, align 8
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %_6)
          to label %bb6 unwind label %cleanup

bb1:                                              ; preds = %bb6
  ret void

bb4:                                              ; preds = %bb3, %cleanup
  %5 = load i64, ptr %_3, align 8
  %_5 = icmp eq i64 %5, %_1.1
  br i1 %_5, label %bb2, label %bb3

cleanup:                                          ; preds = %bb5
  %6 = landingpad { ptr, i32 }
          cleanup
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
  store ptr %7, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %8, ptr %9, align 8
  br label %bb4

bb3:                                              ; preds = %bb4
  %10 = load i64, ptr %_3, align 8
  %_4 = getelementptr inbounds nuw %"alloc::string::String", ptr %_1.0, i64 %10
  %11 = load i64, ptr %_3, align 8
  %12 = add i64 %11, 1
  store i64 %12, ptr %_3, align 8
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %_4) #16
          to label %bb4 unwind label %terminate

bb2:                                              ; preds = %bb4
  %13 = load ptr, ptr %0, align 8
  %14 = getelementptr inbounds i8, ptr %0, i64 8
  %15 = load i32, ptr %14, align 8
  %16 = insertvalue { ptr, i32 } poison, ptr %13, 0
  %17 = insertvalue { ptr, i32 } %16, i32 %15, 1
  resume { ptr, i32 } %17

terminate:                                        ; preds = %bb3
  %18 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %19 = extractvalue { ptr, i32 } %18, 0
  %20 = extractvalue { ptr, i32 } %18, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<u8>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr53drop_in_place$LT$alloc..raw_vec..RawVec$LT$u8$GT$$GT$17h96750253b9c6fdb1E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h929ede4221a7e4c1E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<[std::ffi::os_str::OsString]>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr57drop_in_place$LT$$u5b$std..ffi..os_str..OsString$u5d$$GT$17h871ad3c979847d47E"(ptr align 8 %_1.0, i64 %_1.1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_3 = alloca [8 x i8], align 8
  store i64 0, ptr %_3, align 8
  br label %bb6

bb6:                                              ; preds = %bb5, %start
  %1 = load i64, ptr %_3, align 8
  %_7 = icmp eq i64 %1, %_1.1
  br i1 %_7, label %bb1, label %bb5

bb5:                                              ; preds = %bb6
  %2 = load i64, ptr %_3, align 8
  %_6 = getelementptr inbounds nuw %"std::ffi::os_str::OsString", ptr %_1.0, i64 %2
  %3 = load i64, ptr %_3, align 8
  %4 = add i64 %3, 1
  store i64 %4, ptr %_3, align 8
; invoke core::ptr::drop_in_place<std::ffi::os_str::OsString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17he8bedefff6f040d8E"(ptr align 8 %_6)
          to label %bb6 unwind label %cleanup

bb1:                                              ; preds = %bb6
  ret void

bb4:                                              ; preds = %bb3, %cleanup
  %5 = load i64, ptr %_3, align 8
  %_5 = icmp eq i64 %5, %_1.1
  br i1 %_5, label %bb2, label %bb3

cleanup:                                          ; preds = %bb5
  %6 = landingpad { ptr, i32 }
          cleanup
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
  store ptr %7, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %8, ptr %9, align 8
  br label %bb4

bb3:                                              ; preds = %bb4
  %10 = load i64, ptr %_3, align 8
  %_4 = getelementptr inbounds nuw %"std::ffi::os_str::OsString", ptr %_1.0, i64 %10
  %11 = load i64, ptr %_3, align 8
  %12 = add i64 %11, 1
  store i64 %12, ptr %_3, align 8
; invoke core::ptr::drop_in_place<std::ffi::os_str::OsString>
  invoke void @"_ZN4core3ptr47drop_in_place$LT$std..ffi..os_str..OsString$GT$17he8bedefff6f040d8E"(ptr align 8 %_4) #16
          to label %bb4 unwind label %terminate

bb2:                                              ; preds = %bb4
  %13 = load ptr, ptr %0, align 8
  %14 = getelementptr inbounds i8, ptr %0, i64 8
  %15 = load i32, ptr %14, align 8
  %16 = insertvalue { ptr, i32 } poison, ptr %13, 0
  %17 = insertvalue { ptr, i32 } %16, i32 %15, 1
  resume { ptr, i32 } %17

terminate:                                        ; preds = %bb3
  %18 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %19 = extractvalue { ptr, i32 } %18, 0
  %20 = extractvalue { ptr, i32 } %18, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable
}

; core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17h49baea2715f090f9E"(ptr align 8 %_1) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
; invoke <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
  invoke void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hba90ee8f2e73c280E"(ptr align 8 %_1)
          to label %bb4 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::string::String>>
  invoke void @"_ZN4core3ptr72drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..string..String$GT$$GT$17h0a2891f358d53f36E"(ptr align 8 %_1) #16
          to label %bb1 unwind label %terminate

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb3

bb4:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::string::String>>
  call void @"_ZN4core3ptr72drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..string..String$GT$$GT$17h0a2891f358d53f36E"(ptr align 8 %_1)
  ret void

terminate:                                        ; preds = %bb3
  %5 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %6 = extractvalue { ptr, i32 } %5, 0
  %7 = extractvalue { ptr, i32 } %5, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb1:                                              ; preds = %bb3
  %8 = load ptr, ptr %0, align 8
  %9 = getelementptr inbounds i8, ptr %0, i64 8
  %10 = load i32, ptr %9, align 8
  %11 = insertvalue { ptr, i32 } poison, ptr %8, 0
  %12 = insertvalue { ptr, i32 } %11, i32 %10, 1
  resume { ptr, i32 } %12
}

; core::ptr::drop_in_place<core::option::Option<alloc::string::String>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr70drop_in_place$LT$core..option..Option$LT$alloc..string..String$GT$$GT$17hca4dadf12d2c404cE"(ptr align 8 %_1) unnamed_addr #1 {
start:
  %0 = load i64, ptr %_1, align 8
  %1 = icmp eq i64 %0, -9223372036854775808
  %_2 = select i1 %1, i64 0, i64 1
  %2 = icmp eq i64 %_2, 0
  br i1 %2, label %bb1, label %bb2

bb1:                                              ; preds = %bb2, %start
  ret void

bb2:                                              ; preds = %start
; call core::ptr::drop_in_place<alloc::string::String>
  call void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %_1)
  br label %bb1
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<alloc::string::String>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr72drop_in_place$LT$alloc..raw_vec..RawVec$LT$alloc..string..String$GT$$GT$17h0a2891f358d53f36E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h064cb316ca7dd0ffE"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<alloc::raw_vec::RawVec<std::ffi::os_str::OsString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr77drop_in_place$LT$alloc..raw_vec..RawVec$LT$std..ffi..os_str..OsString$GT$$GT$17h202b337d44986271E"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h8e9c6effcfe9bf22E"(ptr align 8 %_1)
  ret void
}

; core::ptr::drop_in_place<std::rt::lang_start<()>::{{closure}}>
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core3ptr85drop_in_place$LT$std..rt..lang_start$LT$$LP$$RP$$GT$..$u7b$$u7b$closure$u7d$$u7d$$GT$17h33f409949a5f9a1cE"(ptr align 8 %_1) unnamed_addr #0 {
start:
  ret void
}

; core::ptr::drop_in_place<alloc::vec::into_iter::IntoIter<std::ffi::os_str::OsString>>
; Function Attrs: uwtable
define internal void @"_ZN4core3ptr86drop_in_place$LT$alloc..vec..into_iter..IntoIter$LT$std..ffi..os_str..OsString$GT$$GT$17h7aaa73b9490f4a1bE"(ptr align 8 %_1) unnamed_addr #1 {
start:
; call <alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop
  call void @"_ZN86_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hbbaafd7503578276E"(ptr align 8 %_1)
  ret void
}

; core::ptr::non_null::NonNull<T>::offset_from_unsigned
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN4core3ptr8non_null16NonNull$LT$T$GT$20offset_from_unsigned17h914f2be8eb5f0a0dE"(ptr %self, ptr %subtracted) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  br label %bb2

bb2:                                              ; preds = %start
; call core::ptr::const_ptr::<impl *const T>::offset_from_unsigned::precondition_check
  call void @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$20offset_from_unsigned18precondition_check17h86c076201eff20d1E"(ptr %self, ptr %subtracted) #18
  br label %bb4

bb4:                                              ; preds = %bb2
  br label %bb5

bb5:                                              ; preds = %bb4
  br label %bb6

bb6:                                              ; preds = %bb5
  %1 = ptrtoint ptr %self to i64
  %2 = ptrtoint ptr %subtracted to i64
  %3 = sub nuw i64 %1, %2
  %4 = udiv exact i64 %3, 24
  store i64 %4, ptr %0, align 8
  %_0 = load i64, ptr %0, align 8
  ret i64 %_0

bb7:                                              ; No predecessors!
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h755aa7cf85bf3433E(ptr align 1 @alloc_ec595fc0e82ef92fc59bd74f68296eae, i64 73, ptr align 8 @alloc_95ce0164b91fa975d5074664dce8a3bd) #19
  unreachable
}

; core::ptr::const_ptr::<impl *const T>::offset_from_unsigned::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @"_ZN4core3ptr9const_ptr33_$LT$impl$u20$$BP$const$u20$T$GT$20offset_from_unsigned18precondition_check17h86c076201eff20d1E"(ptr %this, ptr %origin) unnamed_addr #3 {
start:
  %_3 = icmp uge ptr %this, %origin
  br i1 %_3, label %bb1, label %bb2

bb2:                                              ; preds = %start
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17hc7163b0cd384d969E(ptr align 1 @alloc_de4e626d456b04760e72bc785ed7e52a, i64 201) #20
  unreachable

bb1:                                              ; preds = %start
  ret void
}

; core::str::<impl str>::parse
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN4core3str21_$LT$impl$u20$str$GT$5parse17h3f57953d3ec15506E"(ptr align 1 %self.0, i64 %self.1) unnamed_addr #0 {
start:
  %0 = alloca [8 x i8], align 8
  %_0 = alloca [8 x i8], align 4
; call core::num::<impl core::str::traits::FromStr for u32>::from_str
  %1 = call i64 @"_ZN4core3num60_$LT$impl$u20$core..str..traits..FromStr$u20$for$u20$u32$GT$8from_str17h0f2039747053d208E"(ptr align 1 %self.0, i64 %self.1)
  store i64 %1, ptr %0, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %_0, ptr align 8 %0, i64 8, i1 false)
  %2 = load i64, ptr %_0, align 4
  ret i64 %2
}

; core::char::methods::<impl char>::to_digit
; Function Attrs: inlinehint uwtable
define internal { i32, i32 } @"_ZN4core4char7methods22_$LT$impl$u20$char$GT$8to_digit17hd0ac33e4739e6dccE"(i32 %self, i32 %radix) unnamed_addr #0 {
start:
  %value = alloca [4 x i8], align 4
  %_6 = alloca [48 x i8], align 8
  %_0 = alloca [8 x i8], align 4
  %_3 = icmp uge i32 %radix, 2
  br i1 %_3, label %bb1, label %bb3

bb3:                                              ; preds = %bb1, %start
  store ptr @alloc_708eb9f2492b697e0d761b647be5d46c, ptr %_6, align 8
  %0 = getelementptr inbounds i8, ptr %_6, i64 8
  store i64 1, ptr %0, align 8
  %1 = load ptr, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, align 8
  %2 = load i64, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, i64 8), align 8
  %3 = getelementptr inbounds i8, ptr %_6, i64 32
  store ptr %1, ptr %3, align 8
  %4 = getelementptr inbounds i8, ptr %3, i64 8
  store i64 %2, ptr %4, align 8
  %5 = getelementptr inbounds i8, ptr %_6, i64 16
  store ptr inttoptr (i64 8 to ptr), ptr %5, align 8
  %6 = getelementptr inbounds i8, ptr %5, i64 8
  store i64 0, ptr %6, align 8
; call core::panicking::panic_fmt
  call void @_ZN4core9panicking9panic_fmt17heec96bfc27e6c546E(ptr align 8 %_6, ptr align 8 @alloc_26977bacf3c1c6974c5960438930cb96) #19
  unreachable

bb1:                                              ; preds = %start
  %_4 = icmp ule i32 %radix, 36
  br i1 %_4, label %bb2, label %bb3

bb2:                                              ; preds = %bb1
  %_8 = icmp ugt i32 %self, 57
  br i1 %_8, label %bb4, label %bb6

bb6:                                              ; preds = %bb4, %bb2
  %7 = sub i32 %self, 48
  store i32 %7, ptr %value, align 4
  br label %bb7

bb4:                                              ; preds = %bb2
  %_9 = icmp ugt i32 %radix, 10
  br i1 %_9, label %bb5, label %bb6

bb5:                                              ; preds = %bb4
  %_11 = sub i32 %self, 65
  %_10 = and i32 %_11, -33
  %8 = add i32 %_10, 10
  store i32 %8, ptr %value, align 4
  br label %bb7

bb7:                                              ; preds = %bb5, %bb6
  %_15 = load i32, ptr %value, align 4
  %_14 = icmp ult i32 %_15, %radix
  br i1 %_14, label %bb8, label %bb9

bb9:                                              ; preds = %bb7
  %9 = load i32, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.1, align 4
  %10 = load i32, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.1, i64 4), align 4
  store i32 %9, ptr %_0, align 4
  %11 = getelementptr inbounds i8, ptr %_0, i64 4
  store i32 %10, ptr %11, align 4
  br label %bb10

bb8:                                              ; preds = %bb7
  %_16 = load i32, ptr %value, align 4
  %12 = getelementptr inbounds i8, ptr %_0, i64 4
  store i32 %_16, ptr %12, align 4
  store i32 1, ptr %_0, align 4
  br label %bb10

bb10:                                             ; preds = %bb8, %bb9
  %13 = load i32, ptr %_0, align 4
  %14 = getelementptr inbounds i8, ptr %_0, i64 4
  %15 = load i32, ptr %14, align 4
  %16 = insertvalue { i32, i32 } poison, i32 %13, 0
  %17 = insertvalue { i32, i32 } %16, i32 %15, 1
  ret { i32, i32 } %17
}

; core::char::methods::encode_utf8_raw_unchecked
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core4char7methods25encode_utf8_raw_unchecked17hff004b846a818601E(i32 %code, ptr %dst) unnamed_addr #0 {
start:
  %len = alloca [8 x i8], align 8
  %_34 = icmp ult i32 %code, 128
  br i1 %_34, label %bb7, label %bb2

bb2:                                              ; preds = %start
  %_35 = icmp ult i32 %code, 2048
  br i1 %_35, label %bb6, label %bb3

bb7:                                              ; preds = %start
  store i64 1, ptr %len, align 8
  %0 = trunc i32 %code to i8
  store i8 %0, ptr %dst, align 1
  br label %bb1

bb3:                                              ; preds = %bb2
  %_36 = icmp ult i32 %code, 65536
  br i1 %_36, label %bb5, label %bb4

bb6:                                              ; preds = %bb2
  store i64 2, ptr %len, align 8
  %_6 = lshr i32 %code, 6
  %_5 = and i32 %_6, 31
  %_4 = trunc i32 %_5 to i8
  %1 = or i8 %_4, -64
  store i8 %1, ptr %dst, align 1
  %_8 = and i32 %code, 63
  %_7 = trunc i32 %_8 to i8
  %_9 = getelementptr inbounds nuw i8, ptr %dst, i64 1
  %2 = or i8 %_7, -128
  store i8 %2, ptr %_9, align 1
  br label %bb1

bb4:                                              ; preds = %bb3
  store i64 4, ptr %len, align 8
  %_22 = lshr i32 %code, 18
  %_21 = and i32 %_22, 7
  %_20 = trunc i32 %_21 to i8
  %3 = or i8 %_20, -16
  store i8 %3, ptr %dst, align 1
  %_25 = lshr i32 %code, 12
  %_24 = and i32 %_25, 63
  %_23 = trunc i32 %_24 to i8
  %_26 = getelementptr inbounds nuw i8, ptr %dst, i64 1
  %4 = or i8 %_23, -128
  store i8 %4, ptr %_26, align 1
  %_29 = lshr i32 %code, 6
  %_28 = and i32 %_29, 63
  %_27 = trunc i32 %_28 to i8
  %_30 = getelementptr inbounds nuw i8, ptr %dst, i64 2
  %5 = or i8 %_27, -128
  store i8 %5, ptr %_30, align 1
  %_32 = and i32 %code, 63
  %_31 = trunc i32 %_32 to i8
  %_33 = getelementptr inbounds nuw i8, ptr %dst, i64 3
  %6 = or i8 %_31, -128
  store i8 %6, ptr %_33, align 1
  br label %bb1

bb5:                                              ; preds = %bb3
  store i64 3, ptr %len, align 8
  %_12 = lshr i32 %code, 12
  %_11 = and i32 %_12, 15
  %_10 = trunc i32 %_11 to i8
  %7 = or i8 %_10, -32
  store i8 %7, ptr %dst, align 1
  %_15 = lshr i32 %code, 6
  %_14 = and i32 %_15, 63
  %_13 = trunc i32 %_14 to i8
  %_16 = getelementptr inbounds nuw i8, ptr %dst, i64 1
  %8 = or i8 %_13, -128
  store i8 %8, ptr %_16, align 1
  %_18 = and i32 %code, 63
  %_17 = trunc i32 %_18 to i8
  %_19 = getelementptr inbounds nuw i8, ptr %dst, i64 2
  %9 = or i8 %_17, -128
  store i8 %9, ptr %_19, align 1
  br label %bb1

bb1:                                              ; preds = %bb7, %bb6, %bb5, %bb4
  ret void
}

; core::hint::assert_unchecked::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core4hint16assert_unchecked18precondition_check17hca21193a5a33f108E(i1 zeroext %cond) unnamed_addr #3 {
start:
  br i1 %cond, label %bb2, label %bb1

bb1:                                              ; preds = %start
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17hc7163b0cd384d969E(ptr align 1 @alloc_64e308ef4babfeb8b6220184de794a17, i64 221) #20
  unreachable

bb2:                                              ; preds = %start
  ret void
}

; core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::RangeInclusive<A>>::next
; Function Attrs: inlinehint uwtable
define internal { i32, i32 } @"_ZN4core4iter5range110_$LT$impl$u20$core..iter..traits..iterator..Iterator$u20$for$u20$core..ops..range..RangeInclusive$LT$A$GT$$GT$4next17h1024345c10d5ac3cE"(ptr align 4 %self) unnamed_addr #0 {
start:
; call <core::ops::range::RangeInclusive<T> as core::iter::range::RangeInclusiveIteratorImpl>::spec_next
  %0 = call { i32, i32 } @"_ZN107_$LT$core..ops..range..RangeInclusive$LT$T$GT$$u20$as$u20$core..iter..range..RangeInclusiveIteratorImpl$GT$9spec_next17h41d95db58e8bf8beE"(ptr align 4 %self)
  %_0.0 = extractvalue { i32, i32 } %0, 0
  %_0.1 = extractvalue { i32, i32 } %0, 1
  %1 = insertvalue { i32, i32 } poison, i32 %_0.0, 0
  %2 = insertvalue { i32, i32 } %1, i32 %_0.1, 1
  ret { i32, i32 } %2
}

; core::iter::traits::exact_size::ExactSizeIterator::len
; Function Attrs: inlinehint uwtable
define internal i64 @_ZN4core4iter6traits10exact_size17ExactSizeIterator3len17hc0998a1072333122E(ptr align 8 %self) unnamed_addr #0 {
start:
  %_9 = alloca [48 x i8], align 8
  %_6 = alloca [16 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  %upper = alloca [16 x i8], align 8
; call <alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::size_hint
  call void @"_ZN103_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h98898cb255338797E"(ptr sret([24 x i8]) align 8 %_3, ptr align 8 %self)
  %lower = load i64, ptr %_3, align 8
  %0 = getelementptr inbounds i8, ptr %_3, i64 8
  %1 = load i64, ptr %0, align 8
  %2 = getelementptr inbounds i8, ptr %0, i64 8
  %3 = load i64, ptr %2, align 8
  store i64 %1, ptr %upper, align 8
  %4 = getelementptr inbounds i8, ptr %upper, i64 8
  store i64 %3, ptr %4, align 8
  %5 = getelementptr inbounds i8, ptr %_6, i64 8
  store i64 %lower, ptr %5, align 8
  store i64 1, ptr %_6, align 8
  %_10 = load i64, ptr %upper, align 8
  %6 = getelementptr inbounds i8, ptr %upper, i64 8
  %7 = load i64, ptr %6, align 8
  %8 = trunc nuw i64 %_10 to i1
  br i1 %8, label %bb6, label %bb5

bb6:                                              ; preds = %start
  %l = getelementptr inbounds i8, ptr %upper, i64 8
  %r = getelementptr inbounds i8, ptr %_6, i64 8
  %9 = getelementptr inbounds i8, ptr %upper, i64 8
  %_13 = load i64, ptr %9, align 8
  %_7 = icmp eq i64 %_13, %lower
  br i1 %_7, label %bb2, label %bb3

bb5:                                              ; preds = %start
  br label %bb3

bb3:                                              ; preds = %bb6, %bb5
  store ptr null, ptr %_9, align 8
; call core::panicking::assert_failed
  call void @_ZN4core9panicking13assert_failed17hb103ed725c5cc5f4E(i8 0, ptr align 8 %upper, ptr align 8 %_6, ptr align 8 %_9, ptr align 8 @alloc_2828be464cbb2e0dbc03a0916e604f3b) #19
  unreachable

bb2:                                              ; preds = %bb6
  ret i64 %lower

bb4:                                              ; No predecessors!
  unreachable
}

; core::iter::traits::iterator::Iterator::collect
; Function Attrs: inlinehint uwtable
define internal void @_ZN4core4iter6traits8iterator8Iterator7collect17hd995fadf1a6bead3E(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_6 = alloca [32 x i8], align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_6, ptr align 8 %self, i64 32, i1 false)
; invoke <alloc::vec::Vec<T> as core::iter::traits::collect::FromIterator<T>>::from_iter
  invoke void @"_ZN95_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$core..iter..traits..collect..FromIterator$LT$T$GT$$GT$9from_iter17h53e7b322d5d68312E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %_6, ptr align 8 @alloc_12d3869f6c326732c79369b6c76af0ca)
          to label %bb1 unwind label %cleanup

bb4:                                              ; preds = %cleanup
  br label %bb2

cleanup:                                          ; preds = %start
  %1 = landingpad { ptr, i32 }
          cleanup
  %2 = extractvalue { ptr, i32 } %1, 0
  %3 = extractvalue { ptr, i32 } %1, 1
  store ptr %2, ptr %0, align 8
  %4 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %3, ptr %4, align 8
  br label %bb4

bb1:                                              ; preds = %start
  ret void

bb2:                                              ; preds = %bb3, %bb4
  %5 = load ptr, ptr %0, align 8
  %6 = getelementptr inbounds i8, ptr %0, i64 8
  %7 = load i32, ptr %6, align 8
  %8 = insertvalue { ptr, i32 } poison, ptr %5, 0
  %9 = insertvalue { ptr, i32 } %8, i32 %7, 1
  resume { ptr, i32 } %9

bb3:                                              ; No predecessors!
; invoke core::ptr::drop_in_place<std::env::Args>
  invoke void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hf4415d48177fd3c1E"(ptr align 8 %self) #16
          to label %bb2 unwind label %terminate

terminate:                                        ; preds = %bb3
  %10 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %11 = extractvalue { ptr, i32 } %10, 0
  %12 = extractvalue { ptr, i32 } %10, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable
}

; core::slice::raw::from_raw_parts::precondition_check
; Function Attrs: inlinehint nounwind uwtable
define internal void @_ZN4core5slice3raw14from_raw_parts18precondition_check17hbef5fd1eeb6d2024E(ptr %data, i64 %size, i64 %align, i64 %len) unnamed_addr #3 personality ptr @rust_eh_personality {
start:
  %0 = alloca [4 x i8], align 4
  %max_len = alloca [8 x i8], align 8
  %_11 = alloca [48 x i8], align 8
  %1 = call i64 @llvm.ctpop.i64(i64 %align)
  %2 = trunc i64 %1 to i32
  store i32 %2, ptr %0, align 4
  %_15 = load i32, ptr %0, align 4
  %3 = icmp eq i32 %_15, 1
  br i1 %3, label %bb8, label %bb9

bb8:                                              ; preds = %start
  %_13 = ptrtoint ptr %data to i64
  %_14 = sub i64 %align, 1
  %_12 = and i64 %_13, %_14
  %4 = icmp eq i64 %_12, 0
  br i1 %4, label %bb6, label %bb7

bb9:                                              ; preds = %start
  store ptr @alloc_e92e94d0ff530782b571cfd99ec66aef, ptr %_11, align 8
  %5 = getelementptr inbounds i8, ptr %_11, i64 8
  store i64 1, ptr %5, align 8
  %6 = load ptr, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, align 8
  %7 = load i64, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, i64 8), align 8
  %8 = getelementptr inbounds i8, ptr %_11, i64 32
  store ptr %6, ptr %8, align 8
  %9 = getelementptr inbounds i8, ptr %8, i64 8
  store i64 %7, ptr %9, align 8
  %10 = getelementptr inbounds i8, ptr %_11, i64 16
  store ptr inttoptr (i64 8 to ptr), ptr %10, align 8
  %11 = getelementptr inbounds i8, ptr %10, i64 8
  store i64 0, ptr %11, align 8
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17heec96bfc27e6c546E(ptr align 8 %_11, ptr align 8 @alloc_8c596b95dca80098cb9680607f401da7) #19
          to label %unreachable unwind label %terminate

bb6:                                              ; preds = %bb8
  %_9 = icmp eq i64 %_13, 0
  %_5 = xor i1 %_9, true
  br i1 %_5, label %bb1, label %bb4

bb7:                                              ; preds = %bb8
  br label %bb4

bb4:                                              ; preds = %bb7, %bb6
  br label %bb5

bb1:                                              ; preds = %bb6
  %_19 = icmp eq i64 %size, 0
  %12 = icmp eq i64 %size, 0
  br i1 %12, label %bb11, label %bb12

bb11:                                             ; preds = %bb1
  store i64 -1, ptr %max_len, align 8
  br label %bb14

bb12:                                             ; preds = %bb1
  br i1 %_19, label %panic, label %bb13

bb14:                                             ; preds = %bb13, %bb11
  %_20 = load i64, ptr %max_len, align 8
  %_7 = icmp ule i64 %len, %_20
  br i1 %_7, label %bb2, label %bb3

bb13:                                             ; preds = %bb12
  %13 = udiv i64 9223372036854775807, %size
  store i64 %13, ptr %max_len, align 8
  br label %bb14

panic:                                            ; preds = %bb12
; invoke core::panicking::panic_const::panic_const_div_by_zero
  invoke void @_ZN4core9panicking11panic_const23panic_const_div_by_zero17h994d37c6832804cbE(ptr align 8 @alloc_7f6e074e6d0c411207076874a56d8dfb) #19
          to label %unreachable unwind label %terminate

terminate:                                        ; preds = %bb9, %panic
  %14 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %15 = extractvalue { ptr, i32 } %14, 0
  %16 = extractvalue { ptr, i32 } %14, 1
; call core::panicking::panic_cannot_unwind
  call void @_ZN4core9panicking19panic_cannot_unwind17he04b9d4fe77d66efE() #17
  unreachable

unreachable:                                      ; preds = %bb9, %panic
  unreachable

bb3:                                              ; preds = %bb14
  br label %bb5

bb2:                                              ; preds = %bb14
  ret void

bb5:                                              ; preds = %bb4, %bb3
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17hc7163b0cd384d969E(ptr align 1 @alloc_a28e8c8fd5088943a8b5d44af697ff83, i64 279) #20
  unreachable
}

; core::result::Result<T,E>::expect
; Function Attrs: inlinehint uwtable
define internal void @"_ZN4core6result19Result$LT$T$C$E$GT$6expect17hc81e96f9cf5b20eaE"(i1 zeroext %self, ptr align 1 %msg.0, i64 %msg.1, ptr align 8 %0) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %e = alloca [0 x i8], align 1
  %_3 = zext i1 %self to i64
  %2 = trunc nuw i64 %_3 to i1
  br i1 %2, label %bb2, label %bb3

bb2:                                              ; preds = %start
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17hfab4c59284c125d5E(ptr align 1 %msg.0, i64 %msg.1, ptr align 1 %e, ptr align 8 @vtable.2, ptr align 8 %0) #19
          to label %unreachable unwind label %cleanup

bb3:                                              ; preds = %start
  ret void

bb4:                                              ; preds = %cleanup
  %3 = load ptr, ptr %1, align 8
  %4 = getelementptr inbounds i8, ptr %1, i64 8
  %5 = load i32, ptr %4, align 8
  %6 = insertvalue { ptr, i32 } poison, ptr %3, 0
  %7 = insertvalue { ptr, i32 } %6, i32 %5, 1
  resume { ptr, i32 } %7

cleanup:                                          ; preds = %bb2
  %8 = landingpad { ptr, i32 }
          cleanup
  %9 = extractvalue { ptr, i32 } %8, 0
  %10 = extractvalue { ptr, i32 } %8, 1
  store ptr %9, ptr %1, align 8
  %11 = getelementptr inbounds i8, ptr %1, i64 8
  store i32 %10, ptr %11, align 8
  br label %bb4

unreachable:                                      ; preds = %bb2
  unreachable

bb1:                                              ; No predecessors!
  unreachable
}

; core::result::Result<T,E>::unwrap_or
; Function Attrs: inlinehint uwtable
define internal i32 @"_ZN4core6result19Result$LT$T$C$E$GT$9unwrap_or17hbcc0c884433cd840E"(i64 %0, i32 %default) unnamed_addr #0 personality ptr @rust_eh_personality {
start:
  %1 = alloca [16 x i8], align 8
  %_0 = alloca [4 x i8], align 4
  %2 = alloca [8 x i8], align 8
  %self = alloca [8 x i8], align 4
  store i64 %0, ptr %2, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %self, ptr align 8 %2, i64 8, i1 false)
  %3 = load i8, ptr %self, align 4
  %4 = trunc nuw i8 %3 to i1
  %_3 = zext i1 %4 to i64
  %5 = trunc nuw i64 %_3 to i1
  br i1 %5, label %bb2, label %bb3

bb2:                                              ; preds = %start
  store i32 %default, ptr %_0, align 4
  br label %bb4

bb3:                                              ; preds = %start
  %6 = getelementptr inbounds i8, ptr %self, i64 4
  %t = load i32, ptr %6, align 4
  store i32 %t, ptr %_0, align 4
  br label %bb4

bb4:                                              ; preds = %bb2, %bb3
  %7 = load i8, ptr %self, align 4
  %8 = trunc nuw i8 %7 to i1
  %_5 = zext i1 %8 to i64
  %9 = trunc nuw i64 %_5 to i1
  br i1 %9, label %bb8, label %bb7

bb5:                                              ; No predecessors!
  %10 = load i8, ptr %self, align 4
  %11 = trunc nuw i8 %10 to i1
  %_6 = zext i1 %11 to i64
  %12 = icmp eq i64 %_6, 0
  br i1 %12, label %bb6, label %bb9

bb6:                                              ; preds = %bb9, %bb5
  %13 = load ptr, ptr %1, align 8
  %14 = getelementptr inbounds i8, ptr %1, i64 8
  %15 = load i32, ptr %14, align 8
  %16 = insertvalue { ptr, i32 } poison, ptr %13, 0
  %17 = insertvalue { ptr, i32 } %16, i32 %15, 1
  resume { ptr, i32 } %17

bb9:                                              ; preds = %bb5
  br label %bb6

bb8:                                              ; preds = %bb4
  br label %bb7

bb7:                                              ; preds = %bb8, %bb4
  %18 = load i32, ptr %_0, align 4
  ret i32 %18

bb1:                                              ; No predecessors!
  unreachable
}

; core::ub_checks::maybe_is_nonoverlapping::runtime
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @_ZN4core9ub_checks23maybe_is_nonoverlapping7runtime17h87f12e8229fe2a2cE(ptr %src, ptr %dst, i64 %size, i64 %count) unnamed_addr #0 {
start:
  %diff = alloca [8 x i8], align 8
  %_9 = alloca [16 x i8], align 8
  %src_usize = ptrtoint ptr %src to i64
  %dst_usize = ptrtoint ptr %dst to i64
  %0 = call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %size, i64 %count)
  %_14.0 = extractvalue { i64, i1 } %0, 0
  %_14.1 = extractvalue { i64, i1 } %0, 1
  br i1 %_14.1, label %bb1, label %bb3

bb3:                                              ; preds = %start
  %1 = getelementptr inbounds i8, ptr %_9, i64 8
  store i64 %_14.0, ptr %1, align 8
  store i64 1, ptr %_9, align 8
  %2 = getelementptr inbounds i8, ptr %_9, i64 8
  %size1 = load i64, ptr %2, align 8
  %_22 = icmp ult i64 %src_usize, %dst_usize
  br i1 %_22, label %bb4, label %bb5

bb1:                                              ; preds = %start
; call core::panicking::panic_nounwind
  call void @_ZN4core9panicking14panic_nounwind17hc7163b0cd384d969E(ptr align 1 @alloc_763310d78c99c2c1ad3f8a9821e942f3, i64 61) #20
  unreachable

bb5:                                              ; preds = %bb3
  %3 = sub i64 %src_usize, %dst_usize
  store i64 %3, ptr %diff, align 8
  br label %bb6

bb4:                                              ; preds = %bb3
  %4 = sub i64 %dst_usize, %src_usize
  store i64 %4, ptr %diff, align 8
  br label %bb6

bb6:                                              ; preds = %bb4, %bb5
  %_11 = load i64, ptr %diff, align 8
  %_0 = icmp uge i64 %_11, %size1
  ret i1 %_0
}

; <str as alloc::string::SpecToString>::spec_to_string
; Function Attrs: inlinehint uwtable
define internal void @"_ZN51_$LT$str$u20$as$u20$alloc..string..SpecToString$GT$14spec_to_string17h14e2aefcfd128bddE"(ptr sret([24 x i8]) align 8 %_0, ptr align 1 %self.0, i64 %self.1) unnamed_addr #0 {
start:
  %bytes = alloca [24 x i8], align 8
; call <T as alloc::slice::<impl [T]>::to_vec_in::ConvertVec>::to_vec
  call void @"_ZN87_$LT$T$u20$as$u20$alloc..slice..$LT$impl$u20$$u5b$T$u5d$$GT$..to_vec_in..ConvertVec$GT$6to_vec17h4913319dabd9188aE"(ptr sret([24 x i8]) align 8 %bytes, ptr align 1 %self.0, i64 %self.1)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %bytes, i64 24, i1 false)
  ret void
}

; <core::fmt::Error as core::fmt::Debug>::fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN53_$LT$core..fmt..Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h705ed12b580d2973E"(ptr align 1 %self, ptr align 8 %f) unnamed_addr #0 {
start:
; call core::fmt::Formatter::write_str
  %_0 = call zeroext i1 @_ZN4core3fmt9Formatter9write_str17h8a65dbcec027f2b5E(ptr align 8 %f, ptr align 1 @alloc_99ac8a81a24cac863217ce4a5cbfabea, i64 5)
  ret i1 %_0
}

; <() as std::process::Termination>::report
; Function Attrs: inlinehint uwtable
define internal i8 @"_ZN54_$LT$$LP$$RP$$u20$as$u20$std..process..Termination$GT$6report17h8c9a1574d0d156d5E"() unnamed_addr #0 {
start:
  ret i8 0
}

; <alloc::string::String as core::fmt::Write>::write_char
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$10write_char17h88aeed3332d36d8bE"(ptr align 8 %self, i32 %c) unnamed_addr #0 {
start:
; call alloc::string::String::push
  call void @_ZN5alloc6string6String4push17hd416b1d063a84370E(ptr align 8 %self, i32 %c)
  ret i1 false
}

; <alloc::string::String as core::fmt::Write>::write_str
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$9write_str17h8c0356c36a6abeafE"(ptr align 8 %self, ptr align 1 %s.0, i64 %s.1) unnamed_addr #0 {
start:
; call alloc::string::String::push_str
  call void @_ZN5alloc6string6String8push_str17h96dc9b7368ef94ceE(ptr align 8 %self, ptr align 1 %s.0, i64 %s.1)
  ret i1 false
}

; alloc::vec::Vec<T,A>::extend_desugared
; Function Attrs: uwtable
define internal void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16extend_desugared17h64a1cac9f103e930E"(ptr align 8 %self, ptr align 8 %iterator, ptr align 8 %0) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %1 = alloca [8 x i8], align 8
  %2 = alloca [16 x i8], align 8
  %src = alloca [24 x i8], align 8
  %_11 = alloca [24 x i8], align 8
  %_9 = alloca [8 x i8], align 8
  %element = alloca [24 x i8], align 8
  %_3 = alloca [24 x i8], align 8
  br label %bb1

bb1:                                              ; preds = %bb8, %start
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::next
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hd466f957c1b1bbd6E"(ptr sret([24 x i8]) align 8 %_3, ptr align 8 %iterator)
          to label %bb2 unwind label %cleanup.loopexit

bb12:                                             ; preds = %bb14, %cleanup
; invoke core::ptr::drop_in_place<std::env::Args>
  invoke void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hf4415d48177fd3c1E"(ptr align 8 %iterator) #16
          to label %bb13 unwind label %terminate

cleanup.loopexit:                                 ; preds = %bb1
  %lpad.loopexit = landingpad { ptr, i32 }
          cleanup
  br label %cleanup

cleanup.loopexit.split-lp:                        ; preds = %bb9
  %lpad.loopexit.split-lp = landingpad { ptr, i32 }
          cleanup
  br label %cleanup

cleanup:                                          ; preds = %cleanup.loopexit.split-lp, %cleanup.loopexit
  %lpad.phi = phi { ptr, i32 } [ %lpad.loopexit, %cleanup.loopexit ], [ %lpad.loopexit.split-lp, %cleanup.loopexit.split-lp ]
  %3 = extractvalue { ptr, i32 } %lpad.phi, 0
  %4 = extractvalue { ptr, i32 } %lpad.phi, 1
  store ptr %3, ptr %2, align 8
  %5 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb12

bb2:                                              ; preds = %bb1
  %6 = load i64, ptr %_3, align 8
  %7 = icmp eq i64 %6, -9223372036854775808
  %_5 = select i1 %7, i64 0, i64 1
  %8 = trunc nuw i64 %_5 to i1
  br i1 %8, label %bb3, label %bb9

bb3:                                              ; preds = %bb2
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %element, ptr align 8 %_3, i64 24, i1 false)
  %9 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %9, align 8
  %_19 = icmp ule i64 %len, 384307168202282325
  br label %bb17

bb9:                                              ; preds = %bb2
; invoke core::ptr::drop_in_place<core::option::Option<alloc::string::String>>
  invoke void @"_ZN4core3ptr70drop_in_place$LT$core..option..Option$LT$alloc..string..String$GT$$GT$17hca4dadf12d2c404cE"(ptr align 8 %_3)
          to label %bb10 unwind label %cleanup.loopexit.split-lp

bb17:                                             ; preds = %bb3
  %self1 = load i64, ptr %self, align 8
  store i64 %self1, ptr %_9, align 8
  br label %bb15

bb16:                                             ; No predecessors!
  store i64 -1, ptr %_9, align 8
  unreachable

bb15:                                             ; preds = %bb17
  %10 = load i64, ptr %_9, align 8
  %_8 = icmp eq i64 %len, %10
  br i1 %_8, label %bb4, label %bb7

bb7:                                              ; preds = %bb15
  br label %bb8

bb4:                                              ; preds = %bb15
; invoke <std::env::Args as core::iter::traits::iterator::Iterator>::size_hint
  invoke void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h60b4dc5d4e8c8713E"(ptr sret([24 x i8]) align 8 %_11, ptr align 8 %iterator)
          to label %bb5 unwind label %cleanup2

bb8:                                              ; preds = %bb6, %bb7
  %11 = getelementptr inbounds i8, ptr %self, i64 8
  %_24 = load ptr, ptr %11, align 8
  %dst = getelementptr inbounds nuw %"alloc::string::String", ptr %_24, i64 %len
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %src, ptr align 8 %element, i64 24, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %dst, ptr align 8 %src, i64 24, i1 false)
  %new_len = add i64 %len, 1
  %12 = getelementptr inbounds i8, ptr %self, i64 16
  store i64 %new_len, ptr %12, align 8
  br label %bb1

bb14:                                             ; preds = %cleanup2
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %element) #16
          to label %bb12 unwind label %terminate

cleanup2:                                         ; preds = %bb5, %bb4
  %13 = landingpad { ptr, i32 }
          cleanup
  %14 = extractvalue { ptr, i32 } %13, 0
  %15 = extractvalue { ptr, i32 } %13, 1
  store ptr %14, ptr %2, align 8
  %16 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %15, ptr %16, align 8
  br label %bb14

bb5:                                              ; preds = %bb4
  %lower = load i64, ptr %_11, align 8
  %17 = call i64 @llvm.uadd.sat.i64(i64 %lower, i64 1)
  store i64 %17, ptr %1, align 8
  %_14 = load i64, ptr %1, align 8
; invoke alloc::vec::Vec<T,A>::reserve
  invoke void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h93ceadd51ac5a80cE"(ptr align 8 %self, i64 %_14, ptr align 8 %0)
          to label %bb6 unwind label %cleanup2

bb6:                                              ; preds = %bb5
  br label %bb8

terminate:                                        ; preds = %bb12, %bb14
  %18 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %19 = extractvalue { ptr, i32 } %18, 0
  %20 = extractvalue { ptr, i32 } %18, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb10:                                             ; preds = %bb9
; call core::ptr::drop_in_place<std::env::Args>
  call void @"_ZN4core3ptr35drop_in_place$LT$std..env..Args$GT$17hf4415d48177fd3c1E"(ptr align 8 %iterator)
  ret void

bb19:                                             ; No predecessors!
  unreachable

bb13:                                             ; preds = %bb12
  %21 = load ptr, ptr %2, align 8
  %22 = getelementptr inbounds i8, ptr %2, i64 8
  %23 = load i32, ptr %22, align 8
  %24 = insertvalue { ptr, i32 } poison, ptr %21, 0
  %25 = insertvalue { ptr, i32 } %24, i32 %23, 1
  resume { ptr, i32 } %25
}

; alloc::vec::Vec<T,A>::len
; Function Attrs: inlinehint uwtable
define internal i64 @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h3514f5503aaa0e8eE"(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 16
  %_0 = load i64, ptr %0, align 8
  %_2 = icmp ule i64 %_0, 384307168202282325
  ret i64 %_0
}

; alloc::vec::Vec<T,A>::reserve
; Function Attrs: uwtable
define internal void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17h93ceadd51ac5a80cE"(ptr align 8 %self, i64 %additional, ptr align 8 %0) unnamed_addr #1 {
start:
  %self1 = alloca [8 x i8], align 8
  %elem_layout = alloca [16 x i8], align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
  store i64 8, ptr %elem_layout, align 8
  %2 = getelementptr inbounds i8, ptr %elem_layout, i64 8
  store i64 24, ptr %2, align 8
  br label %bb6

bb6:                                              ; preds = %start
  %self2 = load i64, ptr %self, align 8
  store i64 %self2, ptr %self1, align 8
  br label %bb4

bb5:                                              ; No predecessors!
  store i64 -1, ptr %self1, align 8
  br label %bb4

bb4:                                              ; preds = %bb6, %bb5
  %3 = load i64, ptr %self1, align 8
  %_10 = sub i64 %3, %len
  %_7 = icmp ugt i64 %additional, %_10
  br i1 %_7, label %bb1, label %bb2

bb2:                                              ; preds = %bb4
  br label %bb3

bb1:                                              ; preds = %bb4
; call alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hb9fbc9e853c7e304E"(ptr align 8 %self, i64 %len, i64 %additional, i64 8, i64 24)
  br label %bb3

bb3:                                              ; preds = %bb1, %bb2
  ret void
}

; alloc::string::String::push
; Function Attrs: inlinehint uwtable
define internal void @_ZN5alloc6string6String4push17hd416b1d063a84370E(ptr align 8 %self, i32 %ch) unnamed_addr #0 {
start:
  %ch_len = alloca [8 x i8], align 8
  %0 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %0, align 8
  %_15 = icmp ule i64 %len, 9223372036854775807
  %_17 = icmp ult i32 %ch, 128
  br i1 %_17, label %bb8, label %bb3

bb3:                                              ; preds = %start
  %_18 = icmp ult i32 %ch, 2048
  br i1 %_18, label %bb7, label %bb4

bb8:                                              ; preds = %start
  store i64 1, ptr %ch_len, align 8
  br label %bb2

bb4:                                              ; preds = %bb3
  %_19 = icmp ult i32 %ch, 65536
  br i1 %_19, label %bb6, label %bb5

bb7:                                              ; preds = %bb3
  store i64 2, ptr %ch_len, align 8
  br label %bb2

bb5:                                              ; preds = %bb4
  store i64 4, ptr %ch_len, align 8
  br label %bb2

bb6:                                              ; preds = %bb4
  store i64 3, ptr %ch_len, align 8
  br label %bb2

bb2:                                              ; preds = %bb8, %bb7, %bb6, %bb5
  %additional = load i64, ptr %ch_len, align 8
; call alloc::vec::Vec<T,A>::reserve
  call void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17hb816475f09642001E"(ptr align 8 %self, i64 %additional, ptr align 8 @alloc_849630626503f5ef0f8eaa4fe7c4e339)
  %1 = getelementptr inbounds i8, ptr %self, i64 8
  %_21 = load ptr, ptr %1, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 16
  %count = load i64, ptr %2, align 8
  %_22 = icmp ule i64 %count, 9223372036854775807
  %_8 = getelementptr inbounds nuw i8, ptr %_21, i64 %count
; call core::char::methods::encode_utf8_raw_unchecked
  call void @_ZN4core4char7methods25encode_utf8_raw_unchecked17hff004b846a818601E(i32 %ch, ptr %_8)
  %_14 = load i64, ptr %ch_len, align 8
  %new_len = add i64 %len, %_14
  %3 = getelementptr inbounds i8, ptr %self, i64 16
  store i64 %new_len, ptr %3, align 8
  ret void
}

; alloc::string::String::as_str
; Function Attrs: inlinehint uwtable
define internal { ptr, i64 } @_ZN5alloc6string6String6as_str17h9083376c2d5a1d92E(ptr align 8 %self) unnamed_addr #0 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_6 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
  br label %bb1

bb1:                                              ; preds = %start
; call core::slice::raw::from_raw_parts::precondition_check
  call void @_ZN4core5slice3raw14from_raw_parts18precondition_check17hbef5fd1eeb6d2024E(ptr %_6, i64 1, i64 1, i64 %len) #18
  br label %bb3

bb3:                                              ; preds = %bb1
  %2 = insertvalue { ptr, i64 } poison, ptr %_6, 0
  %3 = insertvalue { ptr, i64 } %2, i64 %len, 1
  ret { ptr, i64 } %3
}

; alloc::string::String::push_str
; Function Attrs: inlinehint uwtable
define internal void @_ZN5alloc6string6String8push_str17h96dc9b7368ef94ceE(ptr align 8 %self, ptr align 1 %string.0, i64 %string.1) unnamed_addr #0 {
start:
  %_10 = getelementptr inbounds nuw i8, ptr %string.0, i64 %string.1
; call <alloc::vec::Vec<T,A> as alloc::vec::spec_extend::SpecExtend<&T,core::slice::iter::Iter<T>>>::spec_extend
  call void @"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h1c67bd2daaf9978bE"(ptr align 8 %self, ptr %string.0, ptr %_10, ptr align 8 @alloc_8d1458ac186f4f660817c6f55603b839)
  ret void
}

; alloc::raw_vec::RawVecInner<A>::with_capacity_in
; Function Attrs: inlinehint uwtable
define internal { i64, ptr } @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$16with_capacity_in17hacdb32290e29d2efE"(i64 %capacity, i64 %elem_layout.0, i64 %elem_layout.1, ptr align 8 %0) unnamed_addr #0 {
start:
  %self = alloca [8 x i8], align 8
  %elem_layout = alloca [16 x i8], align 8
  %this = alloca [16 x i8], align 8
  %_4 = alloca [24 x i8], align 8
; call alloc::raw_vec::RawVecInner<A>::try_allocate_in
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$15try_allocate_in17h33876ec6b6852a49E"(ptr sret([24 x i8]) align 8 %_4, i64 %capacity, i1 zeroext false, i64 %elem_layout.0, i64 %elem_layout.1)
  %_5 = load i64, ptr %_4, align 8
  %1 = trunc nuw i64 %_5 to i1
  br i1 %1, label %bb3, label %bb4

bb3:                                              ; preds = %start
  %2 = getelementptr inbounds i8, ptr %_4, i64 8
  %err.0 = load i64, ptr %2, align 8
  %3 = getelementptr inbounds i8, ptr %2, i64 8
  %err.1 = load i64, ptr %3, align 8
; call alloc::raw_vec::handle_error
  call void @_ZN5alloc7raw_vec12handle_error17h2860b3dedb861e5cE(i64 %err.0, i64 %err.1, ptr align 8 %0) #19
  unreachable

bb4:                                              ; preds = %start
  %4 = getelementptr inbounds i8, ptr %_4, i64 8
  %5 = load i64, ptr %4, align 8
  %6 = getelementptr inbounds i8, ptr %4, i64 8
  %7 = load ptr, ptr %6, align 8
  store i64 %5, ptr %this, align 8
  %8 = getelementptr inbounds i8, ptr %this, i64 8
  store ptr %7, ptr %8, align 8
  store i64 %elem_layout.0, ptr %elem_layout, align 8
  %9 = getelementptr inbounds i8, ptr %elem_layout, i64 8
  store i64 %elem_layout.1, ptr %9, align 8
  %10 = icmp eq i64 %elem_layout.1, 0
  br i1 %10, label %bb6, label %bb7

bb6:                                              ; preds = %bb4
  store i64 -1, ptr %self, align 8
  br label %bb5

bb7:                                              ; preds = %bb4
  %self1 = load i64, ptr %this, align 8
  store i64 %self1, ptr %self, align 8
  br label %bb5

bb5:                                              ; preds = %bb7, %bb6
  %11 = load i64, ptr %self, align 8
  %_13 = sub i64 %11, 0
  %_8 = icmp ugt i64 %capacity, %_13
  %cond = xor i1 %_8, true
  br label %bb8

bb8:                                              ; preds = %bb5
; call core::hint::assert_unchecked::precondition_check
  call void @_ZN4core4hint16assert_unchecked18precondition_check17hca21193a5a33f108E(i1 zeroext %cond) #18
  br label %bb9

bb9:                                              ; preds = %bb8
  %_0.0 = load i64, ptr %this, align 8
  %12 = getelementptr inbounds i8, ptr %this, i64 8
  %_0.1 = load ptr, ptr %12, align 8
  %13 = insertvalue { i64, ptr } poison, i64 %_0.0, 0
  %14 = insertvalue { i64, ptr } %13, ptr %_0.1, 1
  ret { i64, ptr } %14

bb2:                                              ; No predecessors!
  unreachable
}

; <alloc::string::String as core::fmt::Display>::fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN60_$LT$alloc..string..String$u20$as$u20$core..fmt..Display$GT$3fmt17hbe151b8cb91623eaE"(ptr align 8 %self, ptr align 8 %f) unnamed_addr #0 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_8 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
  br label %bb2

bb2:                                              ; preds = %start
; call core::slice::raw::from_raw_parts::precondition_check
  call void @_ZN4core5slice3raw14from_raw_parts18precondition_check17hbef5fd1eeb6d2024E(ptr %_8, i64 1, i64 1, i64 %len) #18
  br label %bb4

bb4:                                              ; preds = %bb2
; call <str as core::fmt::Display>::fmt
  %_0 = call zeroext i1 @"_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h80e103d61603b646E"(ptr align 1 %_8, i64 %len, ptr align 8 %f)
  ret i1 %_0
}

; <I as core::iter::traits::collect::IntoIterator>::into_iter
; Function Attrs: inlinehint uwtable
define internal void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h86aede69dab3b45bE"(ptr sret([12 x i8]) align 4 %_0, ptr align 4 %self) unnamed_addr #0 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %_0, ptr align 4 %self, i64 12, i1 false)
  ret void
}

; <I as core::iter::traits::collect::IntoIterator>::into_iter
; Function Attrs: inlinehint uwtable
define internal void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h9bcfba1be1fe7363E"(ptr sret([32 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 {
start:
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %self, i64 32, i1 false)
  ret void
}

; <alloc::string::String as core::ops::deref::Deref>::deref
; Function Attrs: inlinehint uwtable
define internal { ptr, i64 } @"_ZN65_$LT$alloc..string..String$u20$as$u20$core..ops..deref..Deref$GT$5deref17hd358e950d7fbf3e3E"(ptr align 8 %self) unnamed_addr #0 {
start:
; call alloc::string::String::as_str
  %0 = call { ptr, i64 } @_ZN5alloc6string6String6as_str17h9083376c2d5a1d92E(ptr align 8 %self)
  %_0.0 = extractvalue { ptr, i64 } %0, 0
  %_0.1 = extractvalue { ptr, i64 } %0, 1
  %1 = insertvalue { ptr, i64 } poison, ptr %_0.0, 0
  %2 = insertvalue { ptr, i64 } %1, i64 %_0.1, 1
  ret { ptr, i64 } %2
}

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hba90ee8f2e73c280E"(ptr align 8 %self) unnamed_addr #1 {
start:
  %0 = getelementptr inbounds i8, ptr %self, i64 8
  %_5 = load ptr, ptr %0, align 8
  %1 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %1, align 8
; call core::ptr::drop_in_place<[alloc::string::String]>
  call void @"_ZN4core3ptr52drop_in_place$LT$$u5b$alloc..string..String$u5d$$GT$17hff8f8410fae3b5cdE"(ptr align 8 %_5, i64 %len)
  ret void
}

; <std::env::Args as core::iter::traits::iterator::Iterator>::size_hint
; Function Attrs: inlinehint uwtable
define internal void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h60b4dc5d4e8c8713E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self) unnamed_addr #0 {
start:
; call <alloc::vec::into_iter::IntoIter<T,A> as core::iter::traits::iterator::Iterator>::size_hint
  call void @"_ZN103_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$9size_hint17h98898cb255338797E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %self)
  ret void
}

; <&mut W as core::fmt::Write::write_fmt::SpecWriteFmt>::spec_write_fmt
; Function Attrs: inlinehint uwtable
define internal zeroext i1 @"_ZN75_$LT$$RF$mut$u20$W$u20$as$u20$core..fmt..Write..write_fmt..SpecWriteFmt$GT$14spec_write_fmt17hb107362395a091c1E"(ptr align 8 %self, ptr align 8 %args) unnamed_addr #0 {
start:
  %0 = alloca [48 x i8], align 8
  %1 = alloca [1 x i8], align 1
  %s = alloca [16 x i8], align 8
  %_3 = alloca [16 x i8], align 8
  %_0 = alloca [1 x i8], align 1
  %_12.0 = load ptr, ptr %args, align 8
  %2 = getelementptr inbounds i8, ptr %args, i64 8
  %_12.1 = load i64, ptr %2, align 8
  %3 = getelementptr inbounds i8, ptr %args, i64 16
  %_13.0 = load ptr, ptr %3, align 8
  %4 = getelementptr inbounds i8, ptr %3, i64 8
  %_13.1 = load i64, ptr %4, align 8
  %5 = icmp eq i64 %_12.1, 0
  br i1 %5, label %bb11, label %bb12

bb11:                                             ; preds = %start
  %6 = icmp eq i64 %_13.1, 0
  br i1 %6, label %bb15, label %bb10

bb12:                                             ; preds = %start
  %7 = icmp eq i64 %_12.1, 1
  br i1 %7, label %bb13, label %bb10

bb15:                                             ; preds = %bb11
  store ptr inttoptr (i64 1 to ptr), ptr %s, align 8
  %8 = getelementptr inbounds i8, ptr %s, i64 8
  store i64 0, ptr %8, align 8
  br label %bb9

bb10:                                             ; preds = %bb13, %bb12, %bb11
  %9 = load ptr, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, align 8
  %10 = load i64, ptr getelementptr inbounds (i8, ptr @anon.2b9cc1fd271beea6e1454a1c8ec7217d.0, i64 8), align 8
  store ptr %9, ptr %s, align 8
  %11 = getelementptr inbounds i8, ptr %s, i64 8
  store i64 %10, ptr %11, align 8
  br label %bb9

bb9:                                              ; preds = %bb10, %bb14, %bb15
  %12 = load ptr, ptr %s, align 8
  %13 = getelementptr inbounds i8, ptr %s, i64 8
  %14 = load i64, ptr %13, align 8
  %15 = ptrtoint ptr %12 to i64
  %16 = icmp eq i64 %15, 0
  %_19 = select i1 %16, i64 0, i64 1
  %_10 = icmp eq i64 %_19, 1
  %17 = call i1 @llvm.is.constant.i1(i1 %_10)
  %18 = zext i1 %17 to i8
  store i8 %18, ptr %1, align 1
  %19 = load i8, ptr %1, align 1
  %_9 = trunc nuw i8 %19 to i1
  br i1 %_9, label %bb7, label %bb8

bb13:                                             ; preds = %bb12
  %20 = icmp eq i64 %_13.1, 0
  br i1 %20, label %bb14, label %bb10

bb14:                                             ; preds = %bb13
  %s1 = getelementptr inbounds nuw { ptr, i64 }, ptr %_12.0, i64 0
  %21 = getelementptr inbounds nuw { ptr, i64 }, ptr %_12.0, i64 0
  %_18.0 = load ptr, ptr %21, align 8
  %22 = getelementptr inbounds i8, ptr %21, i64 8
  %_18.1 = load i64, ptr %22, align 8
  store ptr %_18.0, ptr %s, align 8
  %23 = getelementptr inbounds i8, ptr %s, i64 8
  store i64 %_18.1, ptr %23, align 8
  br label %bb9

bb8:                                              ; preds = %bb9
  br label %bb3

bb7:                                              ; preds = %bb9
  %24 = load ptr, ptr %s, align 8
  %25 = getelementptr inbounds i8, ptr %s, i64 8
  %26 = load i64, ptr %25, align 8
  store ptr %24, ptr %_3, align 8
  %27 = getelementptr inbounds i8, ptr %_3, i64 8
  store i64 %26, ptr %27, align 8
  %28 = load ptr, ptr %_3, align 8
  %29 = getelementptr inbounds i8, ptr %_3, i64 8
  %30 = load i64, ptr %29, align 8
  %31 = ptrtoint ptr %28 to i64
  %32 = icmp eq i64 %31, 0
  %_5 = select i1 %32, i64 0, i64 1
  %33 = trunc nuw i64 %_5 to i1
  br i1 %33, label %bb1, label %bb3

bb3:                                              ; preds = %bb7, %bb8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %0, ptr align 8 %args, i64 48, i1 false)
; call core::fmt::write
  %34 = call zeroext i1 @_ZN4core3fmt5write17h1dbafa36e52e01c5E(ptr align 1 %self, ptr align 8 @vtable.1, ptr align 8 %0)
  %35 = zext i1 %34 to i8
  store i8 %35, ptr %_0, align 1
  br label %bb5

bb1:                                              ; preds = %bb7
  %s.0 = load ptr, ptr %_3, align 8
  %36 = getelementptr inbounds i8, ptr %_3, i64 8
  %s.1 = load i64, ptr %36, align 8
; call <alloc::string::String as core::fmt::Write>::write_str
  %37 = call zeroext i1 @"_ZN58_$LT$alloc..string..String$u20$as$u20$core..fmt..Write$GT$9write_str17h8c0356c36a6abeafE"(ptr align 8 %self, ptr align 1 %s.0, i64 %s.1)
  %38 = zext i1 %37 to i8
  store i8 %38, ptr %_0, align 1
  br label %bb5

bb5:                                              ; preds = %bb3, %bb1
  %39 = load i8, ptr %_0, align 1
  %40 = trunc nuw i8 %39 to i1
  ret i1 %40

bb16:                                             ; No predecessors!
  unreachable
}

; <usize as core::slice::index::SliceIndex<[T]>>::index
; Function Attrs: inlinehint uwtable
define internal align 8 ptr @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17h891b26cc00a09dc3E"(i64 %self, ptr align 8 %slice.0, i64 %slice.1, ptr align 8 %0) unnamed_addr #0 {
start:
  %_4 = icmp ult i64 %self, %slice.1
  br i1 %_4, label %bb1, label %panic

bb1:                                              ; preds = %start
  %_0 = getelementptr inbounds nuw %"alloc::string::String", ptr %slice.0, i64 %self
  ret ptr %_0

panic:                                            ; preds = %start
; call core::panicking::panic_bounds_check
  call void @_ZN4core9panicking18panic_bounds_check17h486908b9a487d47cE(i64 %self, i64 %slice.1, ptr align 8 %0) #19
  unreachable
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h064cb316ca7dd0ffE"(ptr align 8 %self) unnamed_addr #1 {
start:
; call alloc::raw_vec::RawVecInner<A>::deallocate
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h0962effbd9794217E"(ptr align 8 %self, i64 8, i64 24)
  ret void
}

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h8e9c6effcfe9bf22E"(ptr align 8 %self) unnamed_addr #1 {
start:
; call alloc::raw_vec::RawVecInner<A>::deallocate
  call void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h0962effbd9794217E"(ptr align 8 %self, i64 8, i64 24)
  ret void
}

; <alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
; Function Attrs: inlinehint uwtable
define internal align 8 ptr @"_ZN81_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..index..Index$LT$I$GT$$GT$5index17ha322b20bbcd35885E"(ptr align 8 %self, i64 %index, ptr align 8 %0) unnamed_addr #0 {
start:
  %1 = getelementptr inbounds i8, ptr %self, i64 8
  %_6 = load ptr, ptr %1, align 8
  %2 = getelementptr inbounds i8, ptr %self, i64 16
  %len = load i64, ptr %2, align 8
  br label %bb1

bb1:                                              ; preds = %start
; call core::slice::raw::from_raw_parts::precondition_check
  call void @_ZN4core5slice3raw14from_raw_parts18precondition_check17hbef5fd1eeb6d2024E(ptr %_6, i64 24, i64 8, i64 %len) #18
  br label %bb3

bb3:                                              ; preds = %bb1
; call <usize as core::slice::index::SliceIndex<[T]>>::index
  %_0 = call align 8 ptr @"_ZN75_$LT$usize$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$5index17h891b26cc00a09dc3E"(i64 %index, ptr align 8 %_6, i64 %len, ptr align 8 %0)
  ret ptr %_0
}

; <alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
define internal void @"_ZN86_$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hbbaafd7503578276E"(ptr align 8 %self) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %self1 = alloca [8 x i8], align 8
  %guard = alloca [8 x i8], align 8
  store ptr %self, ptr %guard, align 8
  %_6 = load ptr, ptr %guard, align 8
  store ptr %_6, ptr %self1, align 8
  %1 = getelementptr inbounds i8, ptr %_6, i64 8
  %self2 = load ptr, ptr %1, align 8
; invoke core::iter::traits::exact_size::ExactSizeIterator::len
  %len = invoke i64 @_ZN4core4iter6traits10exact_size17ExactSizeIterator3len17hc0998a1072333122E(ptr align 8 %_6)
          to label %bb5 unwind label %cleanup

bb3:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
  invoke void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17ha099effc7cff476aE"(ptr align 8 %guard) #16
          to label %bb4 unwind label %terminate

cleanup:                                          ; preds = %bb5, %start
  %2 = landingpad { ptr, i32 }
          cleanup
  %3 = extractvalue { ptr, i32 } %2, 0
  %4 = extractvalue { ptr, i32 } %2, 1
  store ptr %3, ptr %0, align 8
  %5 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %4, ptr %5, align 8
  br label %bb3

bb5:                                              ; preds = %start
; invoke core::ptr::drop_in_place<[std::ffi::os_str::OsString]>
  invoke void @"_ZN4core3ptr57drop_in_place$LT$$u5b$std..ffi..os_str..OsString$u5d$$GT$17h871ad3c979847d47E"(ptr align 8 %self2, i64 %len)
          to label %bb1 unwind label %cleanup

bb1:                                              ; preds = %bb5
; call core::ptr::drop_in_place<<alloc::vec::into_iter::IntoIter<T,A> as core::ops::drop::Drop>::drop::DropGuard<std::ffi::os_str::OsString,alloc::alloc::Global>>
  call void @"_ZN4core3ptr180drop_in_place$LT$$LT$alloc..vec..into_iter..IntoIter$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$..drop..DropGuard$LT$std..ffi..os_str..OsString$C$alloc..alloc..Global$GT$$GT$17ha099effc7cff476aE"(ptr align 8 %guard)
  ret void

terminate:                                        ; preds = %bb3
  %6 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %7 = extractvalue { ptr, i32 } %6, 0
  %8 = extractvalue { ptr, i32 } %6, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb4:                                              ; preds = %bb3
  %9 = load ptr, ptr %0, align 8
  %10 = getelementptr inbounds i8, ptr %0, i64 8
  %11 = load i32, ptr %10, align 8
  %12 = insertvalue { ptr, i32 } poison, ptr %9, 0
  %13 = insertvalue { ptr, i32 } %12, i32 %11, 1
  resume { ptr, i32 } %13
}

; <T as alloc::slice::<impl [T]>::to_vec_in::ConvertVec>::to_vec
; Function Attrs: inlinehint uwtable
define internal void @"_ZN87_$LT$T$u20$as$u20$alloc..slice..$LT$impl$u20$$u5b$T$u5d$$GT$..to_vec_in..ConvertVec$GT$6to_vec17h4913319dabd9188aE"(ptr sret([24 x i8]) align 8 %_0, ptr align 1 %s.0, i64 %s.1) unnamed_addr #0 {
start:
  %v = alloca [24 x i8], align 8
; call alloc::raw_vec::RawVecInner<A>::with_capacity_in
  %0 = call { i64, ptr } @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$16with_capacity_in17hacdb32290e29d2efE"(i64 %s.1, i64 1, i64 1, ptr align 8 @alloc_2fd90c3491686217d5b4a4abc34a4b75)
  %_10.0 = extractvalue { i64, ptr } %0, 0
  %_10.1 = extractvalue { i64, ptr } %0, 1
  store i64 %_10.0, ptr %v, align 8
  %1 = getelementptr inbounds i8, ptr %v, i64 8
  store ptr %_10.1, ptr %1, align 8
  %2 = getelementptr inbounds i8, ptr %v, i64 16
  store i64 0, ptr %2, align 8
  %3 = getelementptr inbounds i8, ptr %v, i64 8
  %_12 = load ptr, ptr %3, align 8
  br label %bb2

bb2:                                              ; preds = %start
; call core::intrinsics::copy_nonoverlapping::precondition_check
  call void @_ZN4core10intrinsics19copy_nonoverlapping18precondition_check17h3b370664dbbe11e2E(ptr %s.0, ptr %_12, i64 1, i64 1, i64 %s.1) #18
  br label %bb4

bb4:                                              ; preds = %bb2
  %4 = mul i64 %s.1, 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %_12, ptr align 1 %s.0, i64 %4, i1 false)
  %5 = getelementptr inbounds i8, ptr %v, i64 16
  store i64 %s.1, ptr %5, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %_0, ptr align 8 %v, i64 24, i1 false)
  ret void
}

; <alloc::vec::Vec<T> as core::iter::traits::collect::FromIterator<T>>::from_iter
; Function Attrs: inlinehint uwtable
define internal void @"_ZN95_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$core..iter..traits..collect..FromIterator$LT$T$GT$$GT$9from_iter17h53e7b322d5d68312E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iter, ptr align 8 %0) unnamed_addr #0 {
start:
  %_2 = alloca [32 x i8], align 8
; call <I as core::iter::traits::collect::IntoIterator>::into_iter
  call void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h9bcfba1be1fe7363E"(ptr sret([32 x i8]) align 8 %_2, ptr align 8 %iter)
; call <alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter
  call void @"_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h48e8225dbeddda92E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %_2, ptr align 8 %0)
  ret void
}

; <alloc::vec::Vec<T,A> as alloc::vec::spec_extend::SpecExtend<T,I>>::spec_extend
; Function Attrs: uwtable
define internal void @"_ZN97_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$T$C$I$GT$$GT$11spec_extend17h0d29aef41ddbf0c6E"(ptr align 8 %self, ptr align 8 %iter, ptr align 8 %0) unnamed_addr #1 {
start:
; call alloc::vec::Vec<T,A>::extend_desugared
  call void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16extend_desugared17h64a1cac9f103e930E"(ptr align 8 %self, ptr align 8 %iter, ptr align 8 %0)
  ret void
}

; <alloc::vec::Vec<T> as alloc::vec::spec_from_iter::SpecFromIter<T,I>>::from_iter
; Function Attrs: uwtable
define internal void @"_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h48e8225dbeddda92E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iterator, ptr align 8 %0) unnamed_addr #1 {
start:
; call <alloc::vec::Vec<T> as alloc::vec::spec_from_iter_nested::SpecFromIterNested<T,I>>::from_iter
  call void @"_ZN111_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter_nested..SpecFromIterNested$LT$T$C$I$GT$$GT$9from_iter17hd0beb9c273219349E"(ptr sret([24 x i8]) align 8 %_0, ptr align 8 %iterator, ptr align 8 %0)
  ret void
}

; fizzbuzz::parse_args
; Function Attrs: uwtable
define internal i32 @_ZN8fizzbuzz10parse_args17h1742df14b13c5e26E(ptr align 8 %args) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [8 x i8], align 8
  %1 = alloca [8 x i8], align 8
  %2 = alloca [16 x i8], align 8
  %_5 = alloca [8 x i8], align 4
  %_0 = alloca [4 x i8], align 4
; invoke alloc::vec::Vec<T,A>::len
  %_3 = invoke i64 @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$3len17h3514f5503aaa0e8eE"(ptr align 8 %args)
          to label %bb1 unwind label %cleanup

bb9:                                              ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
  invoke void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17h49baea2715f090f9E"(ptr align 8 %args) #16
          to label %bb10 unwind label %terminate

cleanup:                                          ; preds = %bb6, %bb5, %bb4, %bb3, %start
  %3 = landingpad { ptr, i32 }
          cleanup
  %4 = extractvalue { ptr, i32 } %3, 0
  %5 = extractvalue { ptr, i32 } %3, 1
  store ptr %4, ptr %2, align 8
  %6 = getelementptr inbounds i8, ptr %2, i64 8
  store i32 %5, ptr %6, align 8
  br label %bb9

bb1:                                              ; preds = %start
  %_2 = icmp ult i64 %_3, 2
  br i1 %_2, label %bb2, label %bb3

bb3:                                              ; preds = %bb1
; invoke <alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index
  %_7 = invoke align 8 ptr @"_ZN81_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..index..Index$LT$I$GT$$GT$5index17ha322b20bbcd35885E"(ptr align 8 %args, i64 1, ptr align 8 @alloc_2a51b2c50c0c026e4293813ef00e8a1e)
          to label %bb4 unwind label %cleanup

bb2:                                              ; preds = %bb1
  store i32 100, ptr %_0, align 4
  br label %bb7

bb4:                                              ; preds = %bb3
; invoke <alloc::string::String as core::ops::deref::Deref>::deref
  %7 = invoke { ptr, i64 } @"_ZN65_$LT$alloc..string..String$u20$as$u20$core..ops..deref..Deref$GT$5deref17hd358e950d7fbf3e3E"(ptr align 8 %_7)
          to label %bb5 unwind label %cleanup

bb5:                                              ; preds = %bb4
  %_6.0 = extractvalue { ptr, i64 } %7, 0
  %_6.1 = extractvalue { ptr, i64 } %7, 1
; invoke core::str::<impl str>::parse
  %8 = invoke i64 @"_ZN4core3str21_$LT$impl$u20$str$GT$5parse17h3f57953d3ec15506E"(ptr align 1 %_6.0, i64 %_6.1)
          to label %bb6 unwind label %cleanup

bb6:                                              ; preds = %bb5
  store i64 %8, ptr %1, align 8
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %_5, ptr align 8 %1, i64 8, i1 false)
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %0, ptr align 4 %_5, i64 8, i1 false)
  %9 = load i64, ptr %0, align 8
; invoke core::result::Result<T,E>::unwrap_or
  %10 = invoke i32 @"_ZN4core6result19Result$LT$T$C$E$GT$9unwrap_or17hbcc0c884433cd840E"(i64 %9, i32 100)
          to label %bb11 unwind label %cleanup

bb11:                                             ; preds = %bb6
  store i32 %10, ptr %_0, align 4
  br label %bb7

bb7:                                              ; preds = %bb2, %bb11
; call core::ptr::drop_in_place<alloc::vec::Vec<alloc::string::String>>
  call void @"_ZN4core3ptr65drop_in_place$LT$alloc..vec..Vec$LT$alloc..string..String$GT$$GT$17h49baea2715f090f9E"(ptr align 8 %args)
  %11 = load i32, ptr %_0, align 4
  ret i32 %11

terminate:                                        ; preds = %bb9
  %12 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %13 = extractvalue { ptr, i32 } %12, 0
  %14 = extractvalue { ptr, i32 } %12, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb10:                                             ; preds = %bb9
  %15 = load ptr, ptr %2, align 8
  %16 = getelementptr inbounds i8, ptr %2, i64 8
  %17 = load i32, ptr %16, align 8
  %18 = insertvalue { ptr, i32 } poison, ptr %15, 0
  %19 = insertvalue { ptr, i32 } %18, i32 %17, 1
  resume { ptr, i32 } %19
}

; fizzbuzz::fizzbuzz
; Function Attrs: uwtable
define internal void @_ZN8fizzbuzz8fizzbuzz17he3265cd8658c05dcE(ptr sret([24 x i8]) align 8 %_0, i32 %0) unnamed_addr #1 {
start:
  %i = alloca [4 x i8], align 4
  store i32 %0, ptr %i, align 4
  %1 = load i32, ptr %i, align 4
  %_3 = urem i32 %1, 3
  %2 = load i32, ptr %i, align 4
  %_5 = urem i32 %2, 5
  %3 = icmp eq i32 %_3, 0
  br i1 %3, label %bb4, label %bb3

bb4:                                              ; preds = %start
  %4 = icmp eq i32 %_5, 0
  br i1 %4, label %bb8, label %bb7

bb3:                                              ; preds = %start
  %5 = icmp eq i32 %_5, 0
  br i1 %5, label %bb6, label %bb5

bb8:                                              ; preds = %bb4
; call <T as alloc::string::ToString>::to_string
  call void @"_ZN45_$LT$T$u20$as$u20$alloc..string..ToString$GT$9to_string17ha7c54548f8a658e9E"(ptr sret([24 x i8]) align 8 %_0, ptr align 1 @alloc_39d9c0f14cf82e0fad647f0c4a7b4b43, i64 8)
  br label %bb9

bb7:                                              ; preds = %bb4
; call <T as alloc::string::ToString>::to_string
  call void @"_ZN45_$LT$T$u20$as$u20$alloc..string..ToString$GT$9to_string17ha7c54548f8a658e9E"(ptr sret([24 x i8]) align 8 %_0, ptr align 1 @alloc_80a4a3fda200f245ed4df58d4efe532b, i64 4)
  br label %bb9

bb9:                                              ; preds = %bb5, %bb6, %bb7, %bb8
  ret void

bb6:                                              ; preds = %bb3
; call <T as alloc::string::ToString>::to_string
  call void @"_ZN45_$LT$T$u20$as$u20$alloc..string..ToString$GT$9to_string17ha7c54548f8a658e9E"(ptr sret([24 x i8]) align 8 %_0, ptr align 1 @alloc_2de8df279e307344e74413fa3f65e71d, i64 4)
  br label %bb9

bb5:                                              ; preds = %bb3
; call <T as alloc::string::ToString>::to_string
  call void @"_ZN45_$LT$T$u20$as$u20$alloc..string..ToString$GT$9to_string17h21833b828628f54dE"(ptr sret([24 x i8]) align 8 %_0, ptr align 4 %i)
  br label %bb9
}

; fizzbuzz::perform
; Function Attrs: uwtable
define internal void @_ZN8fizzbuzz7perform17h702cd4bd3bf431e9E(i32 %max) unnamed_addr #1 personality ptr @rust_eh_personality {
start:
  %0 = alloca [16 x i8], align 8
  %_19 = alloca [16 x i8], align 8
  %_18 = alloca [16 x i8], align 8
  %_16 = alloca [24 x i8], align 8
  %args = alloca [16 x i8], align 8
  %_13 = alloca [32 x i8], align 8
  %_10 = alloca [48 x i8], align 8
  %i = alloca [4 x i8], align 4
  %_5 = alloca [8 x i8], align 4
  %iter = alloca [12 x i8], align 4
  %_3 = alloca [12 x i8], align 4
  %_2 = alloca [12 x i8], align 4
; call core::ops::range::RangeInclusive<Idx>::new
  call void @"_ZN4core3ops5range25RangeInclusive$LT$Idx$GT$3new17h21c59fb822df068dE"(ptr sret([12 x i8]) align 4 %_3, i32 1, i32 %max)
; call <I as core::iter::traits::collect::IntoIterator>::into_iter
  call void @"_ZN63_$LT$I$u20$as$u20$core..iter..traits..collect..IntoIterator$GT$9into_iter17h86aede69dab3b45bE"(ptr sret([12 x i8]) align 4 %_2, ptr align 4 %_3)
  call void @llvm.memcpy.p0.p0.i64(ptr align 4 %iter, ptr align 4 %_2, i64 12, i1 false)
  br label %bb3

bb3:                                              ; preds = %bb12, %start
; call core::iter::range::<impl core::iter::traits::iterator::Iterator for core::ops::range::RangeInclusive<A>>::next
  %1 = call { i32, i32 } @"_ZN4core4iter5range110_$LT$impl$u20$core..iter..traits..iterator..Iterator$u20$for$u20$core..ops..range..RangeInclusive$LT$A$GT$$GT$4next17h1024345c10d5ac3cE"(ptr align 4 %iter)
  %2 = extractvalue { i32, i32 } %1, 0
  %3 = extractvalue { i32, i32 } %1, 1
  store i32 %2, ptr %_5, align 4
  %4 = getelementptr inbounds i8, ptr %_5, i64 4
  store i32 %3, ptr %4, align 4
  %5 = load i32, ptr %_5, align 4
  %6 = getelementptr inbounds i8, ptr %_5, i64 4
  %7 = load i32, ptr %6, align 4
  %_7 = zext i32 %5 to i64
  %8 = trunc nuw i64 %_7 to i1
  br i1 %8, label %bb6, label %bb7

bb6:                                              ; preds = %bb3
  %9 = getelementptr inbounds i8, ptr %_5, i64 4
  %10 = load i32, ptr %9, align 4
  store i32 %10, ptr %i, align 4
  %11 = load i32, ptr %i, align 4
; call fizzbuzz::fizzbuzz
  call void @_ZN8fizzbuzz8fizzbuzz17he3265cd8658c05dcE(ptr sret([24 x i8]) align 8 %_16, i32 %11)
  store ptr %_16, ptr %args, align 8
  %12 = getelementptr inbounds i8, ptr %args, i64 8
  store ptr %i, ptr %12, align 8
  %13 = getelementptr inbounds i8, ptr %args, i64 8
  %_20 = load ptr, ptr %13, align 8
; invoke core::fmt::rt::Argument::new_display
  invoke void @_ZN4core3fmt2rt8Argument11new_display17h3377736cb327981aE(ptr sret([16 x i8]) align 8 %_18, ptr align 4 %_20)
          to label %bb9 unwind label %cleanup

bb7:                                              ; preds = %bb3
  ret void

bb13:                                             ; preds = %cleanup
; invoke core::ptr::drop_in_place<alloc::string::String>
  invoke void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %_16) #16
          to label %bb14 unwind label %terminate

cleanup:                                          ; preds = %bb11, %bb10, %bb9, %bb6
  %14 = landingpad { ptr, i32 }
          cleanup
  %15 = extractvalue { ptr, i32 } %14, 0
  %16 = extractvalue { ptr, i32 } %14, 1
  store ptr %15, ptr %0, align 8
  %17 = getelementptr inbounds i8, ptr %0, i64 8
  store i32 %16, ptr %17, align 8
  br label %bb13

bb9:                                              ; preds = %bb6
  %_21 = load ptr, ptr %args, align 8
; invoke core::fmt::rt::Argument::new_display
  invoke void @_ZN4core3fmt2rt8Argument11new_display17hef888bca4d0b8c30E(ptr sret([16 x i8]) align 8 %_19, ptr align 8 %_21)
          to label %bb10 unwind label %cleanup

bb10:                                             ; preds = %bb9
  %18 = getelementptr inbounds nuw %"core::fmt::rt::Argument<'_>", ptr %_13, i64 0
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %18, ptr align 8 %_18, i64 16, i1 false)
  %19 = getelementptr inbounds nuw %"core::fmt::rt::Argument<'_>", ptr %_13, i64 1
  call void @llvm.memcpy.p0.p0.i64(ptr align 8 %19, ptr align 8 %_19, i64 16, i1 false)
; invoke core::fmt::rt::<impl core::fmt::Arguments>::new_v1
  invoke void @"_ZN4core3fmt2rt38_$LT$impl$u20$core..fmt..Arguments$GT$6new_v117h725926bb9b040646E"(ptr sret([48 x i8]) align 8 %_10, ptr align 8 @alloc_03a2d32c075c30b45b5a6f468665331f, ptr align 8 %_13)
          to label %bb11 unwind label %cleanup

bb11:                                             ; preds = %bb10
; invoke std::io::stdio::_print
  invoke void @_ZN3std2io5stdio6_print17h068a7c8fe03701f4E(ptr align 8 %_10)
          to label %bb12 unwind label %cleanup

bb12:                                             ; preds = %bb11
; call core::ptr::drop_in_place<alloc::string::String>
  call void @"_ZN4core3ptr42drop_in_place$LT$alloc..string..String$GT$17h602948ec4387f87fE"(ptr align 8 %_16)
  br label %bb3

terminate:                                        ; preds = %bb13
  %20 = landingpad { ptr, i32 }
          filter [0 x ptr] zeroinitializer
  %21 = extractvalue { ptr, i32 } %20, 0
  %22 = extractvalue { ptr, i32 } %20, 1
; call core::panicking::panic_in_cleanup
  call void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() #17
  unreachable

bb14:                                             ; preds = %bb13
  %23 = load ptr, ptr %0, align 8
  %24 = getelementptr inbounds i8, ptr %0, i64 8
  %25 = load i32, ptr %24, align 8
  %26 = insertvalue { ptr, i32 } poison, ptr %23, 0
  %27 = insertvalue { ptr, i32 } %26, i32 %25, 1
  resume { ptr, i32 } %27

bb5:                                              ; No predecessors!
  unreachable
}

; fizzbuzz::main
; Function Attrs: uwtable
define internal void @_ZN8fizzbuzz4main17h833633c1d431bc33E() unnamed_addr #1 {
start:
  %_2 = alloca [32 x i8], align 8
  %args = alloca [24 x i8], align 8
; call std::env::args
  call void @_ZN3std3env4args17hde14f4934c86e2ddE(ptr sret([32 x i8]) align 8 %_2)
; call core::iter::traits::iterator::Iterator::collect
  call void @_ZN4core4iter6traits8iterator8Iterator7collect17hd995fadf1a6bead3E(ptr sret([24 x i8]) align 8 %args, ptr align 8 %_2)
; call fizzbuzz::parse_args
  %max = call i32 @_ZN8fizzbuzz10parse_args17h1742df14b13c5e26E(ptr align 8 %args)
; call fizzbuzz::perform
  call void @_ZN8fizzbuzz7perform17h702cd4bd3bf431e9E(i32 %max)
  ret void
}

; Function Attrs: nounwind uwtable
declare i32 @rust_eh_personality(i32, i32, i64, ptr, ptr) unnamed_addr #4

; <std::env::Args as core::iter::traits::iterator::Iterator>::next
; Function Attrs: uwtable
declare void @"_ZN73_$LT$std..env..Args$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hd466f957c1b1bbd6E"(ptr sret([24 x i8]) align 8, ptr align 8) unnamed_addr #1

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #6

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.uadd.sat.i64(i64, i64) #7

; core::panicking::panic_in_cleanup
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_ZN4core9panicking16panic_in_cleanup17he8958c706877a061E() unnamed_addr #8

; std::rt::lang_start_internal
; Function Attrs: uwtable
declare i64 @_ZN3std2rt19lang_start_internal17ha7d36e169a6a9a91E(ptr align 1, ptr align 8, i64, ptr, i8) unnamed_addr #1

; core::fmt::num::imp::<impl core::fmt::Display for u32>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17h34a78d7dcfa5d09eE"(ptr align 4, ptr align 8) unnamed_addr #1

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.ctpop.i64(i64) #7

; core::panicking::panic_cannot_unwind
; Function Attrs: cold minsize noinline noreturn nounwind optsize uwtable
declare void @_ZN4core9panicking19panic_cannot_unwind17he04b9d4fe77d66efE() unnamed_addr #8

; core::panicking::panic_fmt
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking9panic_fmt17heec96bfc27e6c546E(ptr align 8, ptr align 8) unnamed_addr #9

; core::panicking::panic_nounwind
; Function Attrs: cold noinline noreturn nounwind uwtable
declare void @_ZN4core9panicking14panic_nounwind17hc7163b0cd384d969E(ptr align 1, i64) unnamed_addr #10

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i32, i1 } @llvm.uadd.with.overflow.i32(i32, i32) #7

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i32, i1 } @llvm.umul.with.overflow.i32(i32, i32) #7

; core::num::from_ascii_radix_panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core3num22from_ascii_radix_panic17hdce6039c94e6631fE(i32, ptr align 8) unnamed_addr #9

; <alloc::vec::Vec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
declare void @"_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h3c84f8633605c81bE"(ptr align 8) unnamed_addr #1

; <alloc::raw_vec::RawVec<T,A> as core::ops::drop::Drop>::drop
; Function Attrs: uwtable
declare void @"_ZN77_$LT$alloc..raw_vec..RawVec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h929ede4221a7e4c1E"(ptr align 8) unnamed_addr #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking5panic17h755aa7cf85bf3433E(ptr align 1, i64, ptr align 8) unnamed_addr #9

; core::panicking::assert_failed
; Function Attrs: cold minsize noinline noreturn optsize uwtable
declare void @_ZN4core9panicking13assert_failed17hb103ed725c5cc5f4E(i8, ptr align 8, ptr align 8, ptr align 8, ptr align 8) unnamed_addr #11

; core::panicking::panic_const::panic_const_div_by_zero
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking11panic_const23panic_const_div_by_zero17h994d37c6832804cbE(ptr align 8) unnamed_addr #9

; core::result::unwrap_failed
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core6result13unwrap_failed17hfab4c59284c125d5E(ptr align 1, i64, ptr align 1, ptr align 8, ptr align 8) unnamed_addr #9

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare { i64, i1 } @llvm.umul.with.overflow.i64(i64, i64) #7

; core::fmt::Formatter::write_str
; Function Attrs: uwtable
declare zeroext i1 @_ZN4core3fmt9Formatter9write_str17h8a65dbcec027f2b5E(ptr align 8, ptr align 1, i64) unnamed_addr #1

; alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle
; Function Attrs: cold uwtable
declare void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hb9fbc9e853c7e304E"(ptr align 8, i64, i64, i64, i64) unnamed_addr #12

; alloc::vec::Vec<T,A>::reserve
; Function Attrs: uwtable
declare void @"_ZN5alloc3vec16Vec$LT$T$C$A$GT$7reserve17hb816475f09642001E"(ptr align 8, i64, ptr align 8) unnamed_addr #1

; <alloc::vec::Vec<T,A> as alloc::vec::spec_extend::SpecExtend<&T,core::slice::iter::Iter<T>>>::spec_extend
; Function Attrs: uwtable
declare void @"_ZN132_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$alloc..vec..spec_extend..SpecExtend$LT$$RF$T$C$core..slice..iter..Iter$LT$T$GT$$GT$$GT$11spec_extend17h1c67bd2daaf9978bE"(ptr align 8, ptr, ptr, ptr align 8) unnamed_addr #1

; alloc::raw_vec::RawVecInner<A>::try_allocate_in
; Function Attrs: uwtable
declare void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$15try_allocate_in17h33876ec6b6852a49E"(ptr sret([24 x i8]) align 8, i64, i1 zeroext, i64, i64) unnamed_addr #1

; alloc::raw_vec::handle_error
; Function Attrs: cold minsize noreturn optsize uwtable
declare void @_ZN5alloc7raw_vec12handle_error17h2860b3dedb861e5cE(i64, i64, ptr align 8) unnamed_addr #13

; <str as core::fmt::Display>::fmt
; Function Attrs: uwtable
declare zeroext i1 @"_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h80e103d61603b646E"(ptr align 1, i64, ptr align 8) unnamed_addr #1

; Function Attrs: convergent nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.is.constant.i1(i1) #14

; core::fmt::write
; Function Attrs: uwtable
declare zeroext i1 @_ZN4core3fmt5write17h1dbafa36e52e01c5E(ptr align 1, ptr align 8, ptr align 8) unnamed_addr #1

; core::panicking::panic_bounds_check
; Function Attrs: cold minsize noinline noreturn optsize uwtable
declare void @_ZN4core9panicking18panic_bounds_check17h486908b9a487d47cE(i64, i64, ptr align 8) unnamed_addr #11

; alloc::raw_vec::RawVecInner<A>::deallocate
; Function Attrs: uwtable
declare void @"_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$10deallocate17h0962effbd9794217E"(ptr align 8, i64, i64) unnamed_addr #1

; std::io::stdio::_print
; Function Attrs: uwtable
declare void @_ZN3std2io5stdio6_print17h068a7c8fe03701f4E(ptr align 8) unnamed_addr #1

; std::env::args
; Function Attrs: uwtable
declare void @_ZN3std3env4args17hde14f4934c86e2ddE(ptr sret([32 x i8]) align 8) unnamed_addr #1

define i32 @main(i32 %0, ptr %1) unnamed_addr #15 {
top:
  %2 = sext i32 %0 to i64
; call std::rt::lang_start
  %3 = call i64 @_ZN3std2rt10lang_start17h46345d1e0ea817edE(ptr @_ZN8fizzbuzz4main17h833633c1d431bc33E, i64 %2, ptr %1, i8 0)
  %4 = trunc i64 %3 to i32
  ret i32 %4
}

attributes #0 = { inlinehint uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #1 = { uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #2 = { noinline uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #3 = { inlinehint nounwind uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #4 = { nounwind uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #5 = { cold nounwind uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #6 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #7 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #8 = { cold minsize noinline noreturn nounwind optsize uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #9 = { cold noinline noreturn uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #10 = { cold noinline noreturn nounwind uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #11 = { cold minsize noinline noreturn optsize uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #12 = { cold uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #13 = { cold minsize noreturn optsize uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #14 = { convergent nocallback nofree nosync nounwind willreturn memory(none) }
attributes #15 = { "frame-pointer"="non-leaf" "target-cpu"="apple-m1" }
attributes #16 = { cold }
attributes #17 = { cold noreturn nounwind }
attributes #18 = { nounwind }
attributes #19 = { noreturn }
attributes #20 = { noreturn nounwind }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{i32 7, !"PIE Level", i32 2}
!2 = !{!"rustc version 1.88.0 (6b00bc388 2025-06-23)"}
!3 = !{i64 15756182018324999}
