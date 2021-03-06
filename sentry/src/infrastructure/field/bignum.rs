use std::error::Error;

use tokio_postgres::types::{FromSql, IsNull, ToSql, Type};

use domain::BigNum;

#[derive(Debug)]
pub(crate) struct BigNumPg(BigNum);

impl Into<BigNum> for BigNumPg {
    fn into(self) -> BigNum {
        self.0
    }
}

impl<'a> FromSql<'a> for BigNumPg {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<BigNumPg, Box<dyn Error + Sync + Send>> {
        use std::convert::TryInto;

        let str_slice = <&str as FromSql>::from_sql(ty, raw)?;

        let big_num = str_slice.try_into()?;

        Ok(BigNumPg(big_num))
    }

    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::TEXT | Type::VARCHAR => true,
            _ => false,
        }
    }
}

impl ToSql for BigNumPg {
    fn to_sql(&self, ty: &Type, w: &mut Vec<u8>) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <String as ToSql>::to_sql(&self.0.to_string(), ty, w)
    }

    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::TEXT | Type::VARCHAR => true,
            _ => false,
        }
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut Vec<u8>,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <String as ToSql>::to_sql_checked(&self.0.to_string(), ty, out)
    }
}
