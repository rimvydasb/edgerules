use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use crate::ast::annotations::AnnotationEnum::{DecisionTableAnnotation, ServiceAnnotation};
use crate::ast::token::{EToken, EUnparsedToken};
use crate::ast::token::EToken::Unparsed;
use crate::error_token;
use crate::tokenizer::utils::CharStream;

#[derive(Debug, Clone, PartialEq)]
pub enum EHitPolicy {
    FirstHit,
    MultiHit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EMethod {
    GET
    //POST,
}

impl Display for EHitPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EHitPolicy::FirstHit => write!(f, "\"first-hit\""),
            EHitPolicy::MultiHit => write!(f, "\"multi-hit\""),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationEnum {
    DecisionTableAnnotation(EHitPolicy),
    ServiceAnnotation(EMethod),
    // Todo: Add more annotations: Description (could be useful for NLP)
}

impl AnnotationEnum {
    /// eta_char will be @ that is not needed to be added
    pub fn parse(iter: &mut CharStream) -> EToken {
        let annotation = iter.get_alphanumeric();

        iter.skip_whitespace();

        let args = iter.parse_arguments();

        match annotation.as_str() {
            "DecisionTable" => {
                let hit_policy =
                    if let Some(args) = args {
                        match args.get(0) {
                            Some(hit_policy) => match hit_policy.as_str() {
                                "first-hit" => EHitPolicy::FirstHit,
                                "multi-hit" => EHitPolicy::MultiHit,
                                _ => return error_token!("Invalid hit policy {} for {}", hit_policy,annotation)
                            },
                            None => EHitPolicy::FirstHit,
                        }
                    } else {
                        EHitPolicy::FirstHit
                    };

                Unparsed(EUnparsedToken::Annotation(DecisionTableAnnotation(hit_policy)))
            }
            "Service" => {
                Unparsed(EUnparsedToken::Annotation(ServiceAnnotation(EMethod::GET)))
            }
            _ => {
                return error_token!("Unknown annotation: {}", annotation);
            }
        }
    }

    pub fn get_decision_table(annotations: &Vec<AnnotationEnum>) -> Option<&AnnotationEnum> {
        annotations.iter().find(|&a| match a {
            DecisionTableAnnotation(_) => true,
            _ => false
        })
    }
}

impl Display for AnnotationEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DecisionTableAnnotation(hit_policy) => write!(f, "@DecisionTable(\"{}\")", hit_policy),
            ServiceAnnotation(_method) => write!(f, "@Service")
        }
    }
}

//----------------------------------------------------------------------------------------
// Test Cases
//----------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::test::*;
    use crate::ast::annotations::EHitPolicy::{FirstHit, MultiHit};

    #[test]
    fn test_common() {
        init_logger();

        assert_eq!(
            AnnotationEnum::parse(&mut CharStream::new("DecisionTable(\"first-hit\")")),
            Unparsed(EUnparsedToken::Annotation(DecisionTableAnnotation(FirstHit)))
        );

        assert_ne!(
            AnnotationEnum::parse(&mut CharStream::new("DecisionTable(\"first-hit\")")),
            Unparsed(EUnparsedToken::Annotation(DecisionTableAnnotation(MultiHit)))
        );

        assert_eq!(
            AnnotationEnum::parse(&mut CharStream::new("DecisionTable(\"multi-hit\")")),
            Unparsed(EUnparsedToken::Annotation(DecisionTableAnnotation(MultiHit)))
        );
    }
}