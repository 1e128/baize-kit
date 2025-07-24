use baizekit::app::anyhow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use baizekit::app::new_app;
    use baizekit::component::{DbComponent, LogComponent};

    new_app!()
        .register_component_factory(None::<&str>, LogComponent::new)
        .register_component_factory(None::<&str>, DbComponent::new)
        .run()
        .await
}