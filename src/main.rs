use less_5_task::{Queue, Stack};
// use regex::Regex;
use std::io;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Operator {
    token: char,
    operation: fn(Rational, Rational) -> Rational,
    precedence: u8,
    is_unary: bool,
    is_left_associative: bool,
}

type Rational = f64;
// Типы доступных токенов (лексем)
#[derive(Debug, Copy, Clone, PartialEq)]
enum Token {
    Number(Rational),
    Operator(Operator),
    OpenedParenthesis,
    ClosedParenthesis,
    Function,
    ArgumentSeparator,
}

impl Operator {
    fn new_token(
        token: char,
        precedence: u8,
        is_left_associative: bool,
        is_unary: bool,
        operation: fn(Rational, Rational) -> Rational,
    ) -> Token {
        Token::Operator(Operator {
            token,
            operation,
            precedence,
            is_unary,
            is_left_associative,
        })
    }

    // fn apply(&self, x: Number, y: Number) -> Number {
    //     (self.operation)(x, y)
    // }
}

fn symbol_to_token(x: char) -> Option<Token> {
    let res = match x {
        // '0'..='9' => Token::Number(x.to_digit(10).unwrap() as Rational),
        '+' => Operator::new_token('+', 1, true, true, |x, y| x + y),
        '-' => Operator::new_token('-', 1, true, true, |x, y| x - y),
        '*' => Operator::new_token('*', 2, false, true, |x, y| x * y),
        '/' => Operator::new_token('/', 2, false, true, |x, y| x / y),
        '^' => Operator::new_token('^', 3, false, false, |x, y| x.powf(y)),
        '(' => Token::OpenedParenthesis,
        ')' => Token::ClosedParenthesis,
        ',' => Token::ArgumentSeparator,
        'a'..='z' | 'A'..='Z' => Token::Function,
        _ => return None,
    };

    Some(res)
}

fn test(
    tok: Option<Token>,
    stack: &mut Stack<Token>,
    output: &mut Queue<Token>,
) -> Result<bool, String> {
    let tok = tok.unwrap();
    match tok {
        Token::Number(n) => {
            // Если токен — число, то добавить его в очередь вывода
            output.enqueue(Token::Number(n));
        }
        Token::Function => {
            // Если токен — функция, то поместить его в стек
            stack.push(tok);
        }
        Token::OpenedParenthesis => {
            // Если токен — открывающая скобка, то положить его в стек
            stack.push(tok);
        }
        Token::ArgumentSeparator => {
            // Если токен — разделитель аргументов функции (например запятая):
            //     Пока токен на вершине стека не открывающая скобка:
            //         Переложить оператор из стека в выходную очередь.
            while let Some(top) = stack.peek() {
                if Token::OpenedParenthesis == *top {
                    break;
                }
                output.enqueue(stack.pop().unwrap());
            }
            // Если стек закончился до того, как был встречен токен открывающая скобка,
            //   то в выражении пропущен разделитель аргументов функции (запятая),
            //   либо пропущена открывающая скобка.
            if stack.is_empty() {
                return Err("в выражении пропущен разделитель аргументов функции (запятая), либо пропущена открывающая скобка".to_string());
            }
        }
        Token::Operator(op) => {
            // Если токен — оператор op1, то:
            //     Пока присутствует на вершине стека токен оператор op2,
            //       чей приоритет выше или равен приоритету op1,
            //       и при равенстве приоритетов op1 является левоассоциативным:
            //         Переложить op2 из стека в выходную очередь;
            while let Some(top) = stack.peek() {
                match top {
                    &Token::OpenedParenthesis => break,
                    &Token::Operator(top_op) => {
                        let rhs = top_op.precedence;
                        let lhs = op.precedence;
                        if (rhs > lhs) || (rhs == lhs && op.is_left_associative) {
                            output.enqueue(stack.pop().unwrap());
                        } else {
                            break;
                        }
                    }
                    _ => unreachable!(""),
                }
            }

            // Положить op1 в стек.
            stack.push(tok);
        }
        Token::ClosedParenthesis => {
            // Если токен — закрывающая скобка:
            //     Пока токен на вершине стека не открывающая скобка
            //         Переложить оператор из стека в выходную очередь.
            while let Some(last) = stack.peek() {
                if Token::OpenedParenthesis == *last {
                    break;
                }
                output.enqueue(stack.pop().unwrap());
            }

            // Если стек закончился до того, как был встречен токен открывающая скобка, то в выражении пропущена скобка.
            if stack.is_empty() {
                return Err("в выражении пропущена скобка".to_string());
            }
            // Выкинуть открывающую скобку из стека, но не добавлять в очередь вывода.
            let _ = stack.pop();
            // Если токен на вершине стека — функция, переложить её в выходную очередь.
            if let Some(top) = stack.peek() {
                if Token::Function == *top {
                    output.enqueue(stack.pop().unwrap());
                }
            }
        }
    }

    Ok(true)
}

fn digit_to_token(c: char, q: &mut Queue<Token>) -> Option<Token> {
    let mut f: Rational = c.to_digit(10).unwrap() as Rational;
    if !q.is_empty() {
        f += match q.dequeue() {
            Token::Number(l) => (l * 10.0),
            _ => 0.0,
        };
    }

    q.enqueue(Token::Number(f));

    Some(Token::Number(f))
}

// Процесс преобразования состоит из 3 основных этапов
fn process(input: &String) -> Result<String, String> {
    let mut output: Queue<Token> = Queue::new();
    let mut stack: Stack<Token> = Stack::new();

    let mut position: usize = 0;
    let mut tmp: Rational = 0.0;

    let res: Result<Vec<bool>, String> = input
        .chars()
        .inspect(|_| position += 1)
        .filter(|c| !c.is_whitespace())
        .map(symbol_to_token)
        // .map(|x| -> Option<Token> {
        //     match x {
        //         Ok(tok) => {
        //             if let Token::Number(n) = tok {
        //                 tmp = tmp * 10.0 + n
        //             } else {
        //                 output.enqueue(Token::Number(tmp))
        //             }
        //             Some(tok)
        //         }
        //         Err(_) => None,
        //     }
        // })
        .map(|x| -> Result<bool, String> { test(x, &mut stack, &mut output) })
        .collect();

    println!("check {} {:?}", position, res);
    println!("Output = {:?}", output);
    println!("stack = {:?}", stack);

    Ok("".to_string())

    // // 2. Преобразуем список входных токенов в список в ОПН
    // let output = match convert_to_rpn(tokens) {
    //     Ok(output) => output,
    //     Err(why) => return Err(format!("\r{}", why)),
    // };

    // // 3. Вычисляем результат выражения
    // let result = match calc_and_print(output) {
    //     Ok(result) => format!("\nРезультат: {}", result),
    //     Err(why) => return Err(format!("\r{}", why)),
    // };

    // Ok(result)
}

fn main() {
    print_help();
    loop {
        let stdin = io::stdin();
        let mut input = String::new();
        println!("Введите выражение:");
        stdin
            .read_line(&mut input)
            .expect("Не удалось прочитать строку");
        match process(&input) {
            Ok(result) => println!("{}", result),
            Err(why) => println!("{}", why),
        };

        match request_to_continue() {
            true => continue,
            false => break,
        }
    }
}

fn print_help() {
    println!(
        r#"Данная программа преобразует арифметическую операцию записанную в инфиксной форме в запись обратной польской нотации и вычисляет её.
Поддерживаемые операции:
  унарные:
    '+'
    '-'
  бинарные:
    '+'
    '-'
    '/'
    '*'
 Для выхода нажмите <Ctrl+C>"#
    );
}

fn request_to_continue() -> bool {
    let mut answer = String::new();
    println!("Продолжить (Д/н)");
    io::stdin()
        .read_line(&mut answer)
        .expect("Не удалось прочитать строку");
    match answer.trim() {
        "y" | "Y" | "Д" | "д" => return true,
        "n" | "N" | "Н" | "н" => return false,
        _ => {
            println!("Некорректный ввод. Закрываемся..");
        }
    }

    false
}
