// Copyright 2024 RisingLight Project Authors. Licensed under Apache-2.0.

use std::fmt;
use std::str::FromStr;

use pretty_xmlish::helper::delegate_fmt;
use pretty_xmlish::Pretty;
use serde::{Deserialize, Serialize};

use super::*;
use crate::types::DataType as RlDataType;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct CreateFunction {
    pub schema_name: String,
    pub name: String,
    pub arg_types: Vec<RlDataType>,
    pub arg_names: Vec<String>,
    pub return_type: RlDataType,
    pub language: String,
    pub body: String,
}

impl fmt::Display for CreateFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let explainer = Pretty::childless_record("CreateFunction", self.pretty_function());
        delegate_fmt(&explainer, f, String::with_capacity(1000))
    }
}

impl FromStr for CreateFunction {
    type Err = ();

    fn from_str(_s: &str) -> std::result::Result<Self, Self::Err> {
        Err(())
    }
}

impl CreateFunction {
    pub fn pretty_function<'a>(&self) -> Vec<(&'a str, Pretty<'a>)> {
        vec![
            ("name", Pretty::display(&self.name)),
            ("language", Pretty::display(&self.language)),
            ("body", Pretty::display(&self.body)),
        ]
    }
}

impl Binder {
    pub(super) fn bind_create_function(
        &mut self,
        name: ObjectName,
        args: Option<Vec<OperateFunctionArg>>,
        return_type: Option<DataType>,
        params: CreateFunctionBody,
    ) -> Result {
        let Ok((schema_name, function_name)) = split_name(&name) else {
            return Err(BindError::BindFunctionError(
                "failed to parse the input function name".to_string(),
            ));
        };

        let schema_name = schema_name.to_string();
        let name = function_name.to_string();

        let Some(return_type) = return_type else {
            return Err(BindError::BindFunctionError(
                "`return type` must be specified".to_string(),
            ));
        };
        let return_type = RlDataType::new(DataTypeKind::from(&return_type), false);

        // TODO: language check (e.g., currently only support sql)
        let Some(language) = params.language.clone() else {
            return Err(BindError::BindFunctionError(
                "`language` must be specified".to_string(),
            ));
        };
        let language = language.to_string();

        // SQL udf function supports both single quote (i.e., as 'select $1 + $2')
        // and double dollar (i.e., as $$select $1 + $2$$) for as clause
        let body = match &params.as_ {
            Some(FunctionDefinition::SingleQuotedDef(s)) => s.clone(),
            Some(FunctionDefinition::DoubleDollarDef(s)) => s.clone(),
            None => {
                if params.return_.is_none() {
                    return Err(BindError::BindFunctionError(
                        "AS or RETURN must be specified".to_string(),
                    ));
                }
                // Otherwise this is a return expression
                // Note: this is a current work around, and we are assuming return sql udf
                // will NOT involve complex syntax, so just reuse the logic for select definition
                format!("select {}", &params.return_.unwrap().to_string())
            }
        };

        let mut arg_types = vec![];
        let mut arg_names = vec![];
        for arg in args.unwrap_or_default() {
            arg_types.push(RlDataType::new(DataTypeKind::from(&arg.data_type), false));
            arg_names.push(arg.name.map_or("".to_string(), |n| n.to_string()));
        }

        let f = self.egraph.add(Node::CreateFunction(CreateFunction {
            schema_name,
            name,
            arg_types,
            arg_names,
            return_type,
            language,
            body,
        }));

        Ok(f)
    }
}
