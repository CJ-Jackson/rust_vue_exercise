use crate::dependency::DependencyFlag;

pub struct LoginFlag;

impl DependencyFlag for LoginFlag {
    const ALLOW_USER: bool = false;
    const ALLOW_VISITOR: bool = true;
}

pub struct LogoutFlag;

impl DependencyFlag for LogoutFlag {
    const ALLOW_USER: bool = true;
    const ALLOW_VISITOR: bool = false;
}
