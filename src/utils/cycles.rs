
#[derive(Copy, Clone, Debug)]
pub enum ResetCounter {
    IfCounter = 0,
    ModuloCounter = 1,
    BooleanMulCounter = 2,
    BooleanWrapCounter = 3
}

impl ResetCounter {
    const COUNTER: [ResetCounter; 4] = [Self::IfCounter, Self::ModuloCounter, Self::BooleanMulCounter, Self::BooleanWrapCounter];

    pub fn cardinality() -> usize {
        return Self::COUNTER.len();
    }

    pub fn iter() -> std::slice::Iter<'static, ResetCounter> {
        return Self::COUNTER.iter();
    }

    pub fn into_iter() -> std::array::IntoIter<ResetCounter, 4> {
        return Self::COUNTER.into_iter();
    }

    pub fn from_index(idx: usize) -> ResetCounter {
        return Self::COUNTER[idx];
    }

    pub fn as_slice() -> &'static [ResetCounter] {
        return &Self::COUNTER;
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::IfCounter => return "IfCounter",
            Self::ModuloCounter => return "ModuloCounter",
            Self::BooleanMulCounter => return "BooleanMulCounter",
            Self::BooleanWrapCounter => return "BooleanWrapCounter"
        }
    }

    pub fn get_reset_func(&self) -> Box<dyn Fn(usize, usize) -> usize> {
        return match self {
            Self::IfCounter => {
                Box::new(|idx: usize, reset_number| if idx >= reset_number { 0 } else { idx })
            },

            Self::ModuloCounter => {
                Box::new(|idx: usize, reset_number: usize| idx % reset_number)
            },
            
            Self::BooleanMulCounter => {
                Box::new(|idx: usize, reset_number: usize| idx * !(idx >= reset_number) as usize)
            },

            Self::BooleanWrapCounter => {
                Box::new(|idx: usize, reset_number: usize| idx & 0usize.wrapping_sub(!(idx >= reset_number) as usize))
            }
        }
    }
}