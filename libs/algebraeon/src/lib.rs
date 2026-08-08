mod Mat;
mod NN;
mod Q;
mod ZZ;
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
