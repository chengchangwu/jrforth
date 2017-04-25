extern crate rtforth;

mod vm {
    use rtforth::output::Output;
    use rtforth::jitmem::DataSpace;
    use rtforth::core::{Core, Stack, State, ForwardReferences, Word};
    use rtforth::exception::Exception;

    const BUFFER_SIZE: usize = 0x400;

    pub struct Task {
        state: State,
        s_stk: Stack<isize>,
        r_stk: Stack<isize>,
        f_stk: Stack<f64>,
    }

    pub struct VM {
        current_task: usize,
        tasks: [Task; 2],
        last_error: Option<Exception>,
        structure_depth: usize,
        symbols: Vec<String>,
        last_definition: usize,
        wordlist: Vec<Word<VM>>,
        data_space: DataSpace,
        inbuf: Option<String>,
        tkn: Option<String>,
        outbuf: Option<String>,
        references: ForwardReferences,
        evals: Option<Vec<fn(&mut VM, token: &str)>>,
        evaluation_limit: isize,
    }

    impl VM {
        pub fn new(pages: usize) -> VM {
            let mut vm = VM {
                current_task: 0,
                tasks: [Task {
                            state: State::new(),
                            s_stk: Stack::with_capacity(64),
                            r_stk: Stack::with_capacity(64),
                            f_stk: Stack::with_capacity(16),
                        },
                        Task {
                            state: State::new(),
                            s_stk: Stack::with_capacity(64),
                            r_stk: Stack::with_capacity(64),
                            f_stk: Stack::with_capacity(16),
                        }],
                last_error: None,
                symbols: vec![],
                structure_depth: 0,
                last_definition: 0,
                wordlist: vec![],
                data_space: DataSpace::new(pages),
                inbuf: Some(String::with_capacity(BUFFER_SIZE)),
                tkn: Some(String::with_capacity(64)),
                outbuf: Some(String::with_capacity(BUFFER_SIZE)),
                references: ForwardReferences::new(),
                evals: None,
                evaluation_limit: 80,
            };
            vm.add_core();
            vm.add_output();
            vm.add_primitive("pause", VM::pause);
            vm
        }

        pub fn set_current_task(&mut self, i: usize) {
            self.current_task = i;
        }

        fn pause(&mut self) {
            self.current_task = (self.current_task + 1) % 2;
        }
    }

    impl Core for VM {
        fn last_error(&self) -> Option<Exception> {
            self.last_error
        }
        fn set_error(&mut self, e: Option<Exception>) {
            self.last_error = e;
        }
        fn structure_depth(&self) -> usize {
            self.structure_depth
        }
        fn set_structure_depth(&mut self, depth: usize) {
            self.structure_depth = depth
        }
        fn data_space(&mut self) -> &mut DataSpace {
            &mut self.data_space
        }
        fn data_space_const(&self) -> &DataSpace {
            &self.data_space
        }
        fn output_buffer(&mut self) -> &mut Option<String> {
            &mut self.outbuf
        }
        fn set_output_buffer(&mut self, buffer: String) {
            self.outbuf = Some(buffer);
        }
        fn input_buffer(&mut self) -> &mut Option<String> {
            &mut self.inbuf
        }
        fn set_input_buffer(&mut self, buffer: String) {
            self.inbuf = Some(buffer);
        }
        fn last_token(&mut self) -> &mut Option<String> {
            &mut self.tkn
        }
        fn set_last_token(&mut self, buffer: String) {
            self.tkn = Some(buffer);
        }
        fn s_stack(&mut self) -> &mut Stack<isize> {
            &mut self.tasks[self.current_task].s_stk
        }
        fn r_stack(&mut self) -> &mut Stack<isize> {
            &mut self.tasks[self.current_task].r_stk
        }
        fn f_stack(&mut self) -> &mut Stack<f64> {
            &mut self.tasks[self.current_task].f_stk
        }
        fn symbols_mut(&mut self) -> &mut Vec<String> {
            &mut self.symbols
        }
        fn symbols(&self) -> &Vec<String> {
            &self.symbols
        }
        fn last_definition(&self) -> usize {
            self.last_definition
        }
        fn set_last_definition(&mut self, n: usize) {
            self.last_definition = n;
        }
        fn wordlist_mut(&mut self) -> &mut Vec<Word<Self>> {
            &mut self.wordlist
        }
        fn wordlist(&self) -> &Vec<Word<Self>> {
            &self.wordlist
        }
        fn state(&mut self) -> &mut State {
            &mut self.tasks[self.current_task].state
        }
        fn references(&mut self) -> &mut ForwardReferences {
            &mut self.references
        }
        fn evaluators(&mut self) -> &mut Option<Vec<fn(&mut Self, token: &str)>> {
            &mut self.evals
        }
        fn set_evaluators(&mut self, evaluators: Vec<fn(&mut Self, token: &str)>) {
            self.evals = Some(evaluators)
        }
        fn evaluation_limit(&self) -> isize {
            self.evaluation_limit
        }
    }

    impl Output for VM {}
}

use vm::VM;
use rtforth::core::Core;

fn main() {
    let mut vm = VM::new(0x100);

    vm.set_source(": stars   5 0 do 42 emit pause loop ;");
    vm.evaluate();
    match vm.last_error() {
        Some(e) => {
            println!("{}", e.description());
            vm.reset();
        }
        None => {}
    }

    vm.set_source(": pluses   5 0 do 43 emit pause loop ;");
    vm.evaluate();
    match vm.last_error() {
        Some(e) => {
            println!("{}", e.description());
            vm.reset();
        }
        None => {}
    }

    let stars = vm.find("stars").unwrap();
    vm.set_current_task(0);
    vm.execute_word(stars);
    let pluses = vm.find("pluses").unwrap();
    vm.set_current_task(1);
    vm.execute_word(pluses);
    vm.run();

    match *vm.output_buffer() {
        Some(ref buf) => {
            println!("{}", buf);
        }
        None => {}
    }
}
