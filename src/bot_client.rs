
// Basic structure which represents a Bot inside the library
// Not the same as a discord bot!
pub struct Bot {

    // The discord token for the bot utilised for connecting & accessing the discord api.
    pub token: String,
}

// A Builder for [`Bot`]
// 
// Utilises the builder pattern to help ease users in creating a [`Bot`] due to the complexity of the type.
pub struct BotBuilder {
    pub token: String
}

impl BotBuilder {
    // Creates a new [`BotBuilder`] with the basic required fields as args
    // The rest can be set utilising the methods on this builder or are created when this builder is built.
    pub fn new(token: String) -> Self {
        Self {
            token,
        }
    }
}