use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Ambiguity {
    Ambiguity1,
    CountDownLoop,
    DifferentRequires,
    NewfieldCall,
    NewlineCall,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Await {
    AwaitInSync,
    NotYieldable,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Codestyle {
    CodestyleCheck,
    NameStyleCheck,
    SpellCheck,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Conventions {
    GlobalElement,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Duplicate {
    DuplicateIndex,
    DuplicateSetField,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Global {
    GlobalInNilEnv,
    LowercaseGlobal,
    UndefinedEnvChild,
    UndefinedGlobal,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Luadoc {
    CastTypeMismatch,
    CircleDocClass,
    DocFieldNoClass,
    DuplicateDocAlias,
    DuplicateDocField,
    DuplicateDocParam,
    IncompleteSignatureDoc,
    MissingGlobalDoc,
    MissingLocalExportDoc,
    UndefinedDocClass,
    UndefinedDocName,
    UndefinedDocParam,
    UnknownCastVariable,
    UnknownDiagCode,
    UnknownOperator,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Redefined {
    RedefinedLocal,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Strict {
    CloseNonObject,
    Deprecated,
    DiscardReturns,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Strong {
    NoUnknown,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum TypeCheck {
    AssignTypeMismatch,
    CastLocalType,
    CastTypeMismatch,
    InjectField,
    NeedCheckNil,
    ParamTypeMismatch,
    ReturnTypeMismatch,
    UndefinedField,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Unbalanced {
    MissingFields,
    MissingParameter,
    MissingReturn,
    MissingReturnValue,
    RedundantParameter,
    RedundantReturnValue,
    RedundantValue,
    UnbalancedAssignments,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all="kebab-case")]
pub enum Unused {
    CodeAfterBreak,
    EmptyBlock,
    RedundantReturn,
    TrailingSpace,
    UnreachableCode,
    UnusedFunction,
    UnusedLabel,
    UnusedLocal,
    UnusedVararg,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Diagnostic {
    Ambiguity(Ambiguity),
    Await(Await),
    Codestyle(Codestyle),
    Conventions(Conventions),
    Duplicate(Duplicate),
    Global(Global),
    Luadoc(Luadoc),
    Redefined(Redefined),
    Strict(Strict),
    Strong(Strong),
    TypeCheck(TypeCheck),
    Unbalanced(Unbalanced),
    Unused(Unused),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticGroup {
    Ambiguity,
    Await,
    Codestyle,
    Conventions,
    Duplicate,
    Global,
    Luadoc,
    Redefined,
    Strict,
    Strong,
    TypeCheck,
    Unbalanced,
    Unused,
}
