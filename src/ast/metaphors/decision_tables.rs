use crate::ast::annotations::{AnnotationEnum, EHitPolicy};
use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::expression::StaticLink;
use crate::ast::metaphors::metaphor::Metaphor;
use crate::ast::sequence::CollectionExpression;
use crate::ast::token::EUnparsedToken::FunctionDefinitionLiteral;
use crate::ast::token::ExpressionEnum;
use crate::ast::token::ExpressionEnum::{Collection, Value, Variable};
use crate::ast::utils::array_to_code_sep;
use crate::ast::Link;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::ParseErrorEnum::UnknownError;
use crate::typesystem::errors::{ParseErrorEnum, RuntimeError};
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::utils::intern_field_name;
use log::error;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct TableRow {
    pub condition_cells: Vec<ExpressionEnum>,
    pub action_cells: Vec<ExpressionEnum>,
}

impl TableRow {
    pub fn build(
        condition_cells: Vec<ExpressionEnum>,
        condition_headers: &Vec<ExpressionEnum>,
        action_cells: Vec<ExpressionEnum>,
    ) -> Result<Self, ParseErrorEnum> {
        let re_map = |(_expression, _header): (ExpressionEnum, &ExpressionEnum)| -> Result<ExpressionEnum, ParseErrorEnum>{
            todo!("Implement other comparators than Equals")

            // if expression.get_type() == ValueType::BooleanType {
            //     Ok(expression)
            // } else {
            //     todo!("Implement other comparators than Equals")
            //     //Ok(ExpressionEnum::Operator(Rc::new(ComparatorOperator::build(Equals, header, expression)?)))
            // }
        };

        // let (expressions, errors) : (Vec<ExpressionEnum>,Vec<ParseErrorEnum>) = condition_cells.into_iter()
        //     .zip(condition_headers)
        //     .map(re_map)
        //     .partition(Result::is_ok);
        //     //.collect::<Vec<Result<ExpressionEnum,ParseErrorEnum>>>();

        let mut cells = Vec::new();
        for result in condition_cells
            .into_iter()
            .zip(condition_headers)
            .map(re_map)
            .collect::<Vec<Result<ExpressionEnum, ParseErrorEnum>>>()
        {
            cells.push(result?);
        }

        Ok(TableRow {
            condition_cells: cells,
            action_cells,
        })
    }

    #[allow(dead_code)]
    pub fn evaluate(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<bool, RuntimeError> {
        let mut all_conditions_met = true;
        for condition in &self.condition_cells {
            let result = condition.eval(Rc::clone(&context))?;
            if let ValueEnum::BooleanValue(value) = result {
                if !value {
                    all_conditions_met = false;
                    break;
                }
            } else {
                return RuntimeError::eval_error(format!(
                    "Condition must return boolean value, got {}",
                    result
                ))
                .into();
            }
        }

        Ok(all_conditions_met)
    }
}

impl Display for TableRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let row = self
            .condition_cells
            .iter()
            .chain(self.action_cells.iter())
            .map(|s| format!("{}", s))
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "[{}]", row)
    }
}

#[derive(Debug, PartialEq)]
pub struct DecisionTable {
    pub table_name: String,
    pub hit_policy: EHitPolicy,
    pub rows: Vec<TableRow>,
    pub arguments: Vec<FormalParameter>,
    pub inputs: Vec<ExpressionEnum>,
    pub output: Vec<String>,
}

impl DecisionTable {
    pub fn build(
        annotations: Vec<AnnotationEnum>,
        table_name: String,
        arguments: Vec<FormalParameter>,
        rows: CollectionExpression,
    ) -> Result<Self, ParseErrorEnum> {
        let inputs: Vec<ExpressionEnum> = Vec::new();
        let mut output = Vec::new();
        let mut table_rows = Vec::new();
        let mut hit_policy = EHitPolicy::FirstHit;
        let mut length = 0;
        for annotation in annotations {
            if let AnnotationEnum::DecisionTableAnnotation(policy) = annotation {
                hit_policy = policy
            }
        }

        let mut get_headers = true;

        for row in rows.elements {
            match row {
                Collection(mut columns) => {
                    if get_headers {
                        get_headers = false;

                        length = columns.elements.len();

                        if arguments.len() > columns.elements.len() {
                            return Err(UnknownError(format!(
                                "There are more arguments ({}) than columns ({})",
                                arguments.len(),
                                columns.elements.len()
                            )));
                        }

                        if arguments.len() == columns.elements.len() {
                            return Err(UnknownError(format!(
                                "No action columns exists for {} columns length table",
                                arguments.len()
                            )));
                        }

                        let (_condition_cells, action_cells) =
                            columns.elements.split_at(arguments.len());

                        //inputs = condition_cells.to_vec();

                        for expression in action_cells {
                            match expression {
                                Value(ValueEnum::StringValue(StringEnum::String(header_name))) => {
                                    output.push(header_name.clone())
                                }
                                Variable(variable) => output.push(variable.get_name()),
                                _ => {
                                    return Err(UnknownError(
                                        "Invalid header name, only string values are supported"
                                            .to_string(),
                                    ))
                                }
                            }
                        }
                    } else {
                        if columns.elements.len() != length {
                            return Err(UnknownError(format!(
                                "Row length {} does not match header length {}",
                                columns.elements.len(),
                                length
                            )));
                        }

                        let action_cells = columns.elements.split_off(inputs.len());

                        let table_row = TableRow::build(columns.elements, &inputs, action_cells)?;

                        table_rows.push(table_row);
                    }
                }
                _ => {
                    return Err(UnknownError(
                        "Invalid row format, only collections are supported".to_string(),
                    ));
                }
            }
        }

        Ok(DecisionTable {
            table_name,
            hit_policy,
            rows: table_rows,
            arguments,
            inputs,
            output,
        })
    }

    #[allow(dead_code)]
    fn evaluate(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<bool, RuntimeError> {
        // for each table rows
        for row in &self.rows {
            // evaluate row and see if it matches
            let all_conditions_met = row.evaluate(Rc::clone(&context))?;

            if all_conditions_met {
                for (action, header) in row.action_cells.iter().zip(self.output.iter()) {
                    let result = action.eval(Rc::clone(&context));
                    context
                        .borrow()
                        .stack_insert(intern_field_name(header), result);
                }

                return Ok(true);
            }
        }

        for _header in self.output.iter() {
            todo!()
            //let err = Err(RuntimeError::EvalError(header.clone(), context.borrow().get_assigned_to_field()));
            //context.borrow_mut().stack.insert(header.clone(), err);
        }

        Ok(false)
    }
}

impl Display for DecisionTable {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        let annotation = AnnotationEnum::DecisionTableAnnotation(self.hit_policy.clone());
        let function = FunctionDefinitionLiteral(
            vec![annotation],
            self.table_name.clone(),
            self.arguments.clone(),
        );

        let headers = self
            .inputs
            .iter()
            .map(|header| format!("{}", header))
            .chain(self.output.iter().map(|header| header.to_string()));

        let rows = self
            .rows
            .iter()
            .map(|row| format!("{}", row))
            .collect::<Vec<String>>()
            .join(",\n");

        Display::fmt(&function, _f)
            .and(write!(_f, ":"))
            .and(write!(_f, "["))
            .and(write!(_f, "[{}],", array_to_code_sep(headers, ", ")))
            .and(write!(_f, "{}", rows))
            .and(write!(_f, "]"))
    }
}

// @Todo: implement
impl StaticLink for DecisionTable {
    fn link(&mut self, _ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        error!("Not implemented yet");

        Ok(ValueType::BooleanType)
    }
}

impl TypedValue for DecisionTable {
    fn get_type(&self) -> ValueType {
        todo!()
    }
}

impl Metaphor for DecisionTable {
    fn get_name(&self) -> String {
        self.table_name.clone()
    }

    fn get_parameters(&self) -> &Vec<FormalParameter> {
        &self.arguments
    }

    fn create_context(&self, _arguments: Vec<FormalParameter>) -> Link<FunctionContext> {
        todo!()
    }

    // fn create_eval_context(&self, input_values: Vec<Result<ValueEnum, RuntimeError>>) -> Result<Rc<RefCell<ExecutionContext>>, RuntimeError> {
    //
    //     let mut ctx = ExecutionContext::create_for(ContextObjectBuilder::new().build());
    //
    //     for (value, argument) in input_values.into_iter().zip(self.get_parameters()) {
    //         ctx.stack.insert(argument.name.clone(), value);
    //     }
    //
    //     let execution_context = Rc::new(RefCell::new(ctx));
    //
    //     self.evaluate(Rc::clone(&execution_context))?;
    //
    //     Ok(execution_context)
    // }
}

//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use log::info;

    use crate::ast::context::context_object_type::FormalParameter;

    use crate::ast::token::ExpressionEnum;
    use crate::utils::test::*;

    #[allow(dead_code)]
    type E = ExpressionEnum;
    #[allow(dead_code)]
    type Arg = FormalParameter;

    #[test]
    fn test_first() {
        init_logger();
    }

    #[allow(dead_code)]
    fn test_common() {
        init_logger();

        info!(">>> test_common()");

        // let table = DecisionTable::build(
        //     vec![AnnotationEnum::DecisionTableAnnotation(EHitPolicy::FirstHit)],
        //     "test".to_string(),
        //     vec![Arg::from("application : Application"), Arg::from("age : number")],
        //     vec![
        //         E::from(vec![E::variable("application.status"), E::variable("age"), E::variable("eligibility")]),
        //         E::from(vec![E::from("NEW"), E::from(5.0), E::from(true)]),
        //         E::from(vec![E::from("OLD"), E::from(4.0), E::from(false)]),
        //     ],
        // ).unwrap();
        //
        // assert_eq!(table.table_name, "test");
        // assert_eq!(table.hit_policy, EHitPolicy::FirstHit);
        // assert_eq!(table.rows.len(), 2);
        // assert_eq!(table.arguments.len(), 2);
        // assert_eq!(table.inputs.len(), 2);
        // assert_eq!(table.output.len(), 1);
        //
        // // first execution
        //
        // let mut b = ContextObjectBuilder::new();
        // b.add("status", E::from("NEW"));
        // b.add("type", E::from("APPL"));
        // b.add("age", E::from(88));
        //
        // let _application1 = ExecutionContext::create_for(b.build()).to_rc();

        // let result1 = table.create_eval_context(vec![
        //     Ok(ValueEnum::from(application1)),
        //     Ok(ValueEnum::from(5.0)),
        // ]);
        //
        // match result1 {
        //     Ok(_obj) => {
        //         //assert_eq!(Finder::find_and_link(obj, "eligibility"), Ok(BooleanValue(true)));
        //         todo!("test_common()");
        //     }
        //     Err(error) => {
        //         panic!("Error: {:?}", error);
        //     }
        // }

        // second execution

        // let mut b = ContextObjectBuilder::new();
        // b.add("status", E::from("NEW"));
        // b.add("type", E::from("APPL"));
        // b.add("age", E::from(88));
        //
        // let application2 = b.build();
        //
        // let result2 = table.create_eval_context(vec![
        //     Ok(ValueEnum::from(application2)),
        //     Ok(ValueEnum::from(777.0)),
        // ]);
        //
        // // @Todo: NotFound SV should be returned
        // match result2 {
        //     Ok(obj) => {
        //         assert_eq!(Finder::find_and_eval(obj, "eligibility"), Err(Right(RuntimeFieldNotFound("eligibility".to_string(), "#in_function".to_string()))));
        //     }
        //     Err(error) => {
        //         panic!("Error: {:?}", error);
        //     }
        // }
    }
}
