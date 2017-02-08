//! Standard slog-rs extensions.
#![warn(missing_docs)]

extern crate slog;
extern crate thread_local;

use slog::Drain;

use std::sync::{mpsc, Mutex};
use std::fmt;
use std::{io, thread};
use slog::{Record, RecordStatic, Level, SingleKV};
use slog::{Serializer, OwnedKVList};


/// `Async` drain
///
/// `Async` will send all the logging records to a wrapped drain running in another thread.
///
/// Note: Dropping `Async` waits for it's worker-thread to finish (thus handle all previous
/// requests). If you can't tolerate the delay, make sure you drop `Async` drain instance eg. in
/// another thread.
pub struct Async {
    ref_sender: Mutex<mpsc::Sender<AsyncMsg>>,
    tl_sender: thread_local::ThreadLocal<mpsc::Sender<AsyncMsg>>,
    join: Mutex<Option<thread::JoinHandle<()>>>,
}

impl Async {
    /// Create `Async` drain
    ///
    /// The wrapped drain must handle all error conditions (`Drain<Error=Never>`). See
    /// `slog::DrainExt::fuse()` and `slog::DrainExt::ignore_err()` for typical error handling
    /// strategies.
    pub fn new<D: slog::Drain<Error=slog::Never> + Send + 'static>(drain: D) -> Self {
        let (tx, rx) = mpsc::channel();
        let join = thread::spawn(move || {
                loop {
                    match rx.recv().unwrap() {
                        AsyncMsg::Record(r) => {
                            let rs = RecordStatic {
                                level: r.level,
                                file: r.file,
                                line: r.line,
                                column: r.column,
                                function: r.function,
                                module: r.module,
                                target: &r.target,
                            };
                            // Idea here is, that because the representation of
                            // `[Box<KV>]` and `[&KV]` are the same, the optimizer
                            // can turn this into NOP.
                            let record_values: Vec<&slog::KV> = r.record_values
                                .iter()
                                .map(|kv| (&**kv as &slog::KV))
                                .collect();

                            drain.log(
                                &Record::new(&rs,
                                             format_args!("{}", r.msg),
                                             record_values.as_slice()
                                            ),
                                            &r.logger_values
                                            ).unwrap();
                        }
                        AsyncMsg::Finish => return,
                    }
                }
        });

        Async{
            ref_sender: Mutex::new(tx),
            tl_sender: thread_local::ThreadLocal::new(),
            join: Mutex::new(Some(join)),
        }
    }

    fn get_sender(&self) -> &mpsc::Sender<AsyncMsg> {
        self.tl_sender.get_or(|| {
            // TODO: Change to `get_or_try` https://github.com/Amanieu/thread_local-rs/issues/2
            Box::new(self.ref_sender.lock().unwrap().clone())
        })
    }

    /// Send `AsyncRecord` to a worker thread.
    fn send(&self, r: AsyncRecord) -> io::Result<()> {
        let sender = self.get_sender();

        sender.send(AsyncMsg::Record(r))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Send failed"))
    }

}

type RecordValues = Vec<Box<slog::KV+Send>>;

struct ToSendSerializer {
    record_values: RecordValues,
}

impl ToSendSerializer {
    fn new() -> Self {
        ToSendSerializer { record_values: Vec::new() }
    }

    fn finish(self) -> RecordValues {
        self.record_values
    }
}

impl Serializer for ToSendSerializer {
    fn emit_bool(&mut self, key: &str, val: bool) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_unit(&mut self, key: &str) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), ())));
        Ok(())
    }
    fn emit_none(&mut self, key: &str) -> slog::Result {
        let val: Option<()> = None;
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_char(&mut self, key: &str, val: char) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_u8(&mut self, key: &str, val: u8) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_i8(&mut self, key: &str, val: i8) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_u16(&mut self, key: &str, val: u16) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_i16(&mut self, key: &str, val: i16) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_u32(&mut self, key: &str, val: u32) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_i32(&mut self, key: &str, val: i32) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_f32(&mut self, key: &str, val: f32) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_u64(&mut self, key: &str, val: u64) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_i64(&mut self, key: &str, val: i64) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_f64(&mut self, key: &str, val: f64) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_usize(&mut self, key: &str, val: usize) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_isize(&mut self, key: &str, val: isize) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
    fn emit_str(&mut self, key: &str, val: &str) -> slog::Result {
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val.to_owned())));
        Ok(())
    }
    fn emit_arguments(&mut self, key: &str, val: &fmt::Arguments) -> slog::Result {
        let val = fmt::format(*val);
        self.record_values.push(Box::new(SingleKV(key.to_owned(), val)));
        Ok(())
    }
}


impl Drain for Async {
    type Error = io::Error;

    fn log(&self, record: &Record, logger_values: &OwnedKVList) -> io::Result<()> {

        let mut ser = ToSendSerializer::new();
        for kv in record.values() {
            try!(kv.serialize(record, &mut ser))
        }

        self.send(AsyncRecord {
            msg: fmt::format(record.msg()),
            level: record.level(),
            file: record.file(),
            line: record.line(),
            column: record.column(),
            function: record.function(),
            module: record.module(),
            target: String::from(record.target()),
            logger_values: logger_values.clone(),
            record_values: ser.finish(),
        })
    }
}

struct AsyncRecord {
    msg: String,
    level: Level,
    file: &'static str,
    line: u32,
    column: u32,
    function: &'static str,
    module: &'static str,
    target: String,
    logger_values: OwnedKVList,
    record_values: RecordValues,
}

enum AsyncMsg {
    Record(AsyncRecord),
    Finish,
}

impl Drop for Async {
    fn drop(&mut self) {
        let sender = self.get_sender();

        let _ = sender.send(AsyncMsg::Finish);
        let _ = self.join.lock().unwrap().take().unwrap().join();
    }
}
