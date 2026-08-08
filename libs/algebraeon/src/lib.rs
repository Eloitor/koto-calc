mod Alg;
mod CF;
mod FF;
mod Ideal;
mod Mat;
mod NN;
mod Poly;
mod Q;
mod Quat;
mod ZZ;
use algebraeon_rings::num_theory::{
    JacobiSymbolError, LegendreSymbolError, jacobi_symbol, kronecker_symbol, legendre_symbol,
};
use koto_runtime::prelude::*;

pub fn version_string() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn make_module() -> KMap {
    let result = KMap::default();

    let mut nn = KMap::default();
    nn.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| match ctx.args() {
            [] => NN::NN::generator(ctx),
            [KValue::Number(n)] => Ok(NN::NN::make_koto_object(*n).into()),
            unexpected => unexpected_args("|Number|", unexpected),
        })
        .into(),
    );

    nn.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("NN".into())).into(),
    );
    nn.add_fn("primes", |_ctx| {
        Ok(KValue::Object(KObject::from(
            NN::NNPrimesIterator::new(),
        )))
    });

    result.insert("NN", nn);

    let mut zz = KMap::default();

    zz.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| match ctx.args() {
            [KValue::Number(n)] => Ok(ZZ::ZZ::make_koto_object(*n).into()),
            unexpected => unexpected_args("|Number|", unexpected),
        })
        .into(),
    );

    zz.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("ZZ".into())).into(),
    );

    result.insert("ZZ", zz);

    let mut q = KMap::default();

    q.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| match ctx.args() {
            [num] => Ok(Q::Q::rational_from_value(num).map(Q::Q).map(KObject::from)?.into()),
            [num, den] => {
                let numerator = Q::Q::rational_from_value(num)?;
                let denominator = Q::Q::rational_from_value(den)?;
                if denominator == algebraeon::nzq::Rational::ZERO {
                    return runtime_error!("Q: denominator must not be zero");
                }
                Ok(KValue::Object(KObject::from(Q::Q(numerator / denominator))))
            }
            unexpected => unexpected_args("|Number| or |Number, Number|", unexpected),
        })
        .into(),
    );

    q.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("Q".into())).into(),
    );

    result.insert("Q", q);
    let mut poly = KMap::default();

    poly.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| match ctx.args() {
            [KValue::List(list)] => Ok(Poly::Poly::from_koto_list(list)?.into()),
            unexpected => unexpected_args("|List|", unexpected),
        })
        .into(),
    );

    poly.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("Poly".into())).into(),
    );

    result.insert("Poly", poly);


    let mut mat = KMap::default();

    mat.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| match ctx.args() {
            [list] => Ok(Mat::Mat::from_value(list).map(KObject::from)?.into()),
            unexpected => unexpected_args("|List of lists|", unexpected),
        })
        .into(),
    );

    mat.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("Mat".into())).into(),
    );

    result.insert("Mat", mat);

    let mut quat = KMap::default();

    quat.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| Quat::Quat::from_args(ctx.args())).into(),
    );

    quat.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("Quat".into())).into(),
    );

    result.insert("Quat", quat);

    let mut alg = KMap::default();

    alg.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| Alg::Alg::from_args(ctx.args())).into(),
    );

    alg.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("Alg".into())).into(),
    );

    result.insert("Alg", alg);

    let mut ideal = KMap::default();

    ideal.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| Ideal::Ideal::from_args(ctx.args())).into(),
    );

    ideal.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("Ideal".into())).into(),
    );

    result.insert("Ideal", ideal);

    let mut zzn = KMap::default();

    zzn.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| Ideal::ZZn::from_args(ctx.args())).into(),
    );

    zzn.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("ZZn".into())).into(),
    );

    result.insert("ZZn", zzn);

    let mut ff = KMap::default();

    ff.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| FF::FF::from_args(ctx.args())).into(),
    );

    ff.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("FF".into())).into(),
    );

    result.insert("FF", ff);

    let mut cf = KMap::default();

    cf.insert_meta(
        MetaKey::Call,
        KNativeFunction::new(|ctx| CF::CF::from_args(ctx.args())).into(),
    );

    cf.insert_meta(
        MetaKey::UnaryOp(UnaryOp::Display),
        KNativeFunction::new(|_ctx| Ok("CF".into())).into(),
    );

    cf.add_fn("periodic", |ctx| match ctx.args() {
        [KValue::List(initial), KValue::List(repeats)] => CF::CF::periodic(initial, repeats),
        unexpected => unexpected_args("|List, List|", unexpected),
    });

    result.insert("CF", cf);

    result.add_fn("legendre", |ctx| match ctx.args() {
        [a, p] => {
            let a = CF::integer_from_value(a)?;
            let p = CF::natural_from_value(p)?;
            match legendre_symbol(&a, &p) {
                Ok(value) => Ok(KValue::Object(KObject::from(crate::ZZ::ZZ::from_integer(
                    value.into(),
                )))),
                Err(LegendreSymbolError::BottomNotOddPrime) => {
                    runtime_error!("legendre: the bottom argument must be an odd prime, got {}", p)
                }
            }
        }
        unexpected => unexpected_args("|a, p|", unexpected),
    });

    result.add_fn("jacobi", |ctx| match ctx.args() {
        [a, n] => {
            let a = CF::integer_from_value(a)?;
            let n = CF::natural_from_value(n)?;
            match jacobi_symbol(&a, &n) {
                Ok(value) => Ok(KValue::Object(KObject::from(crate::ZZ::ZZ::from_integer(
                    value.into(),
                )))),
                Err(JacobiSymbolError::BottomEven) => {
                    runtime_error!("jacobi: the bottom argument must be odd, got {}", n)
                }
            }
        }
        unexpected => unexpected_args("|a, n|", unexpected),
    });

    result.add_fn("kronecker", |ctx| match ctx.args() {
        [a, n] => {
            let a = CF::integer_from_value(a)?;
            let n = CF::integer_from_value(n)?;
            let value = kronecker_symbol(&a, &n);
            Ok(KValue::Object(KObject::from(crate::ZZ::ZZ::from_integer(
                value.into(),
            ))))
        }
        unexpected => unexpected_args("|a, n|", unexpected),
    });

    result.add_fn("eulers_constant", |_ctx| {
        Ok(KValue::Object(KObject::from(CF::CF::eulers_constant())))
    });

    result.add_fn("gcd", |ctx| match ctx.args() {
        [KValue::Object(n), KValue::Object(m)] => {
            // Check if both objects are NN::NN
            if let (Ok(nn_n), Ok(nn_m)) = (n.cast::<crate::NN::NN>(), m.cast::<crate::NN::NN>()) {
                // Call the gcd function on the Natural values
                let result_natural = algebraeon::nzq::gcd(nn_n.0.clone(), nn_m.0.clone());
                let result_nn = KObject::from(crate::NN::NN(result_natural));
                Ok(result_nn.into())
            } else {
                // If not both NN::NN, return an error with the original arguments
                unexpected_args("|NN, NN|", &[n.clone().into(), m.clone().into()])
            }
        }
        unexpected => unexpected_args("|Object, Object|", unexpected),
    });

    result.add_fn("lcm", |ctx| match ctx.args() {
        [KValue::Object(n), KValue::Object(m)] => {
            // Check if both objects are NN::NN
            if let (Ok(nn_n), Ok(nn_m)) = (n.cast::<crate::NN::NN>(), m.cast::<crate::NN::NN>()) {
                // Call the gcd function on the Natural values
                let result_natural = algebraeon::nzq::lcm(nn_n.0.clone(), nn_m.0.clone());
                let result_nn = KObject::from(crate::NN::NN(result_natural));
                Ok(result_nn.into())
            } else {
                // If not both NN::NN, return an error with the original arguments
                unexpected_args("|NN, NN|", &[n.clone().into(), m.clone().into()])
            }
        }
        unexpected => unexpected_args("|Object, Object|", unexpected),
    });

    result
}
