use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn subscribe(ctx: Context<'_>) -> Result<(), Error> {
    if ctx
        .data()
        .subscribers
        .lock()
        .unwrap()
        .contains(&ctx.author().id)
    {
        ctx.say("You are already subscribed.").await?;
    } else {
        ctx.data()
            .subscribers
            .lock()
            .unwrap()
            .insert(ctx.author().id);
        ctx.say("You are now subscribed.").await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn unsubscribe(ctx: Context<'_>) -> Result<(), Error> {
    if ctx
        .data()
        .subscribers
        .lock()
        .unwrap()
        .contains(&ctx.author().id)
    {
        ctx.data()
            .subscribers
            .lock()
            .unwrap()
            .remove(&ctx.author().id);
        ctx.say("You have been unsubscribed.").await?;
    } else {
        ctx.say("You are not currently subscribed.").await?;
    }
    Ok(())
}
