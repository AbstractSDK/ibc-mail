use crate::{
    contract::{App, ClientResult},
    msg::AppMigrateMsg,
    CLIENT_FEATURES,
};
use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use ibcmail::client::msg::ClientInstantiateMsg;
use ibcmail::client::state::FEATURES;

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    _msg: ClientInstantiateMsg,
) -> ClientResult {
    for feature in CLIENT_FEATURES {
        FEATURES.save(deps.storage, feature.to_string(), &true)?;
    }

    Ok(app
        .response("instantiate")
        .add_attribute("features", CLIENT_FEATURES.join(",")))
}
