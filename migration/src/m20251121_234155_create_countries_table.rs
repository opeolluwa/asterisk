use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("countries")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(string_len("currency_code", 10).not_null())
                    .col(string_len("currency", 100).not_null())
                    .col(string_len("country", 100).not_null())
                    .col(text("flag").null())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"INSERT INTO countries (identifier, currency_code, currency, country, flag) VALUES
            ('01KAMCMRDRK9WTZNJEH3D8J2MT', 'AED', 'UAE Dirham', 'United Arab Emirates', NULL),
            ('01KAMCMRDS8YR95XPACRDW36X3', 'AFN', 'Afghan Afghani', 'Afghanistan', NULL),
            ('01KAMCMRDTT5T3SWKF5CCZE35V', 'ALL', 'Albanian Lek', 'Albania', NULL),
            ('01KAMCMRDT0S19XH321RPJ3SRJ', 'AMD', 'Armenian Dram', 'Armenia', NULL),
            ('01KAMCMRDT50VJQWWGJXTMCWRF', 'ANG', 'Netherlands Antillian Guilder', 'Netherlands Antilles', NULL),
            ('01KAMCMRDTEHS4QE83010QFVHN', 'AOA', 'Angolan Kwanza', 'Angola', NULL),
            ('01KAMCMRDTMB3J9G5DC1FBK0S5', 'ARS', 'Argentine Peso', 'Argentina', NULL),
            ('01KAMCMRDTM3NF4KMAQTSYMFXB', 'AUD', 'Australian Dollar', 'Australia', NULL),
            ('01KAMCMRDTWHPSXTYD2J1X9DCJ', 'AWG', 'Aruban Florin', 'Aruba', NULL),
            ('01KAMCMRDTWSB98GGFGSJ2E4N4', 'AZN', 'Azerbaijani Manat', 'Azerbaijan', NULL),
            ('01KAMCMRDTP99H9RCMEQ2ZDARV', 'BAM', 'Bosnia and Herzegovina Mark', 'Bosnia and Herzegovina', NULL),
            ('01KAMCMRDTKKZ93WY9G51V189J', 'BBD', 'Barbados Dollar', 'Barbados', NULL),
            ('01KAMCMRDT04C616AYDJC57PFY', 'BDT', 'Bangladeshi Taka', 'Bangladesh', NULL),
            ('01KAMCMRDT0G89RQGKNCEG9B92', 'BGN', 'Bulgarian Lev', 'Bulgaria', NULL),
            ('01KAMCMRDT1QK2GW6S2HC08TSF', 'BHD', 'Bahraini Dinar', 'Bahrain', NULL),
            ('01KAMCMRDT9BG37HZKX115JS9Z', 'BIF', 'Burundian Franc', 'Burundi', NULL),
            ('01KAMCMRDT1M3KPW6E7XKA3SER', 'BMD', 'Bermudian Dollar', 'Bermuda', NULL),
            ('01KAMCMRDTG22QWXDMRAYJKTJ7', 'BND', 'Brunei Dollar', 'Brunei Darussalam', NULL),
            ('01KAMCMRDTV10NDVW8K16VVG5G', 'BOB', 'Bolivian Boliviano', 'Bolivia', NULL),
            ('01KAMCMRDT19A55MVXZZYQ4XWF', 'BRL', 'Brazilian Real', 'Brazil', NULL),
            ('01KAMCMRDTGJYY9M20BZTN5881', 'BSD', 'Bahamian Dollar', 'Bahamas', NULL),
            ('01KAMCMRDT71DW7M8G8B2HYMA7', 'BTN', 'Bhutanese Ngultrum', 'Bhutan', NULL),
            ('01KAMCMRDTV7TK27PFP96V7K0N', 'BWP', 'Botswana Pula', 'Botswana', NULL),
            ('01KAMCMRDTMR27B0YRV2WMTVXA', 'BTN', 'Belarusian Ruble', 'Belarus', NULL),
            ('01KAMCMRDTZRCD2P3NVSRVK9HH', 'BZD', 'Belize Dollar', 'Belize', NULL),
            ('01KAMCMRDT2VB15H7YQE7VERC3', 'CAD', 'Canadian Dollar', 'Canada', NULL),
            ('01KAMCMRDTQNJY5HYJYWMGMF6V', 'CDF', 'Congolese Franc', 'Democratic Republic of the Congo', NULL),
            ('01KAMCMRDTR6ZYE8MP007Y9RPV', 'CHF', 'Swiss Franc', 'Switzerland', NULL),
            ('01KAMCMRDT0G17QGYRZZDW3FPF', 'CLP', 'Chilean Peso', 'Chile', NULL),
            ('01KAMCMRDTFZAW6Z5J6DX1S6FW', 'CNY', 'Chinese Renminbi', 'China', NULL),
            ('01KAMCMRDT74GPB1GKATWAN83W', 'COP', 'Colombian Peso', 'Colombia', NULL),
            ('01KAMCMRDTB4FDS1VZVVRDPYD2', 'CRC', 'Costa Rican Colon', 'Costa Rica', NULL),
            ('01KAMCMRDTR7Z6HME5D7F68Z42', 'CUP', 'Cuban Peso', 'Cuba', NULL),
            ('01KAMCMRDVDJS2GV4ZCAA1E18T', 'CVE', 'Cape Verdean Escudo', 'Cape Verde', NULL),
            ('01KAMCMRDVKYGP4J6AC3KE4A4J', 'CZK', 'Czech Koruna', 'Czech Republic', NULL)
            ON CONFLICT DO NOTHING;"#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("countries").to_owned())
            .await
    }
}
