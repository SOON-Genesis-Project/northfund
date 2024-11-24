use std::mem::size_of;

use anchor_lang::prelude::*;

declare_id!("GzTAeUQX8Fdzyw99wEYC6iqY3bWKBPVgGftQeHM7D3YB");

#[program]
pub mod northfund {
    use anchor_lang::system_program;

    use super::*;

    pub fn create_campaign (
        ctx: Context<CreateCampaign>,

         email:String,
         title:String,
         name: String,

         admission_proof_url: String, 
         university_name: String, 

         matric_number: String,
         course_of_study: String,
         year_of_entry: u16,
    
         student_image_url: String,
         student_result_image_url: String,
    
         funding_reason: String,
         project_link: String,
    
         goal: u64,
         start_at: i64,
         end_at: i64,

    ) -> Result<()> {

        let campaign = &mut ctx.accounts.campaign;
        let clock = Clock::get().unwrap();
        let current_timestamp = clock.unix_timestamp;

        require!(start_at >= current_timestamp, Errors::StartTimeEarly);
        require!(end_at > start_at, Errors::EndTimeSmall);
        require!(goal > 0, Errors::GoalZero);

        campaign.admin = ctx.accounts.signer.key(); 
        campaign.donation_completed = false;
        campaign.claimed = false;
        
        campaign.email = email;
        campaign.title = title.clone();

        campaign.name = name;
        campaign.admission_proof_url = admission_proof_url;
        campaign.university_name = university_name;

        campaign.matric_number = matric_number;
        campaign.course_of_study = course_of_study;
        campaign.year_of_entry = year_of_entry;
        campaign.student_image_url = student_image_url;
        campaign.student_result_image_url = student_result_image_url;
        campaign.funding_reason = funding_reason;
        campaign.project_link = project_link;
        campaign.goal = goal;
        campaign.total_donated = 0;
        campaign.start_at = start_at;
        campaign.end_at = end_at;

        msg!("campaign created, {}", title);
    
        Ok(())

    }

    pub fn cancel_campaign (ctx:Context<CancelCampaign>) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;

        let clock = Clock::get();
        let current_timestamp = clock.unwrap().unix_timestamp;

        require!(current_timestamp < campaign.start_at, Errors::CampaignStarted);

        msg!("campaign cancelled: {}", campaign.key().to_string());

        Ok(())
    }

    pub fn donate (ctx:Context<Donate>, amount:u64) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let contribution = &mut ctx.accounts.contribution;

        let clock = Clock::get();
        let current_timestamp = clock.unwrap().unix_timestamp;

        require!(current_timestamp >= campaign.start_at, Errors::CampaignNotStarted);
        require!(current_timestamp <= campaign.end_at, Errors::CampaignOver);
        require!(campaign.donation_completed == false, Errors::DonationCompleted);
        require!(amount > 0, Errors::AmountZero);

        let remaining_amount = campaign.goal - campaign.total_donated;
        let amount = if amount > remaining_amount {remaining_amount} else {amount};

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: campaign.to_account_info().clone(),
            }
        );

        system_program::transfer(cpi_context, amount)?;
        campaign.total_donated += amount;
        contribution.campaign = campaign.key();
        contribution.admin = ctx.accounts.signer.key();
        contribution.amount += amount;

        if campaign.total_donated >= campaign.goal {
            campaign.donation_completed = true;
        }

        msg!("new donation for campaign: {}, amount:{}", campaign.key().to_string(), amount);


        Ok(())
    }

    pub fn claim_donation (ctx: Context<ClaimDonation>) -> Result<()>{
        let campaign = &mut ctx.accounts.campaign;

        require!(campaign.donation_completed == true, Errors::DonationNotCompleted);
        require!(campaign.claimed  == false, Errors::DonationsClaimed);

        campaign.sub_lamports(campaign.total_donated)?;
        ctx.accounts.admin.add_lamports(campaign.total_donated)?;

        campaign.claimed = true;
        msg!("donations claimed for campaign:  {}", campaign.key().to_string());

        Ok(())

    }

    pub fn cancel_donation (ctx:Context<CancelDonation>) -> Result<()> {
        let campaign = &mut ctx.accounts.campaign;
        let contribution = &mut ctx.accounts.contribution;

        let clock = Clock::get();
        let current_timestamp = clock.unwrap().unix_timestamp;

        require!(current_timestamp > campaign.end_at, Errors::CampaignNotOver);
        require!(campaign.donation_completed == false, Errors::DonationCompleted);

        let amount: u64 = contribution.amount;
        campaign.sub_lamports(amount)?;
        ctx.accounts.admin.add_lamports(amount)?;

        msg!("donation cancelled for campaign:  {}", campaign.key().to_string());

        Ok(())
    }


}

#[derive(Accounts)]
#[instruction( email:String, title:String, name: String, admission_proof_url:String, university_name:String,  matric_number: String, course_of_study: String, year_of_entry: u16, student_image_url: String, student_result_image_url: String, project_link: String, funding_reason: String)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + // Account discriminator
        32 + // Pubkey size
        2 +  // Two bool values (donation_completed and claimed)
        2 +  // u16 year_of_entry size
        4 + email.len() + 
        4 + title.len() + 
        4 + name.len() + 
        4 + admission_proof_url.len() + 
        4 + university_name.len() + 

        4 + matric_number.len() + 
        4 + course_of_study.len() + 
        4 + student_image_url.len() + 
        4 + student_result_image_url.len() + 
        4 + project_link.len() + 
        4 + funding_reason.len() +
        8 + // goal (u64)
        8 + // total_donated (u64)
        8 + // start_at (i64)
        8,  // end_at (i64)
        seeds = [matric_number.as_bytes(), signer.key().as_ref()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelCampaign<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, has_one = admin @ Errors::SignerIsNotAuthority)]
    pub campaign : Account<'info, Campaign>
}


#[derive(Accounts)]
pub struct Donate <'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(
        init_if_needed,
        payer = signer,
        space = size_of::<Contribution>() + 8,
        seeds = [campaign.key().as_ref(), signer.key().as_ref()],
        bump
    )]

    pub contribution : Account<'info, Contribution>,
    pub system_program : Program<'info, System>
}


#[derive(Accounts)]

pub struct ClaimDonation <'info> {
    #[account(mut)]
    pub admin : Signer<'info>,

    #[account(mut, has_one = admin @ Errors::SignerIsNotAuthority)]
    pub campaign: Account<'info, Campaign>
}

#[derive(Accounts)]
pub struct CancelDonation <'info> {
    #[account(mut)]
    pub admin : Signer<'info>,

    #[account(mut)]
    pub campaign : Account<'info, Campaign>,

    #[account(mut, close = admin , has_one = admin @ Errors::SignerIsNotAuthority)]
    pub contribution : Account<'info, Contribution>,

    pub system_program: Program<'info, System>
}


#[account]
pub struct Campaign {
    pub admin: Pubkey,

    pub donation_completed: bool,
    pub claimed: bool,

    pub email: String,
    pub title: String,
    pub name: String,

    pub admission_proof_url: String, 
    pub university_name: String, 

    pub matric_number: String,
    pub course_of_study: String,
    pub year_of_entry: u16,

    pub student_image_url: String,
    pub student_result_image_url: String,

    pub funding_reason: String,
    pub project_link: String,

    pub goal: u64,
    pub total_donated: u64,
    pub start_at: i64,
    pub end_at: i64,
}

#[account]
pub struct Contribution {
    pub campaign: Pubkey,
    pub admin: Pubkey,
    pub amount: u64,
}


#[error_code]
pub enum Errors {
    #[msg("start time is too early")]
    StartTimeEarly,
    #[msg("end time is too small")]
    EndTimeSmall,
    #[msg("goal must be greater than zero")]
    GoalZero,
    #[msg("amount must be greater than zero")]
    AmountZero,
    #[msg("campaign is not started")]
    CampaignNotStarted,
    #[msg("campaign has already started")]
    CampaignStarted,
    #[msg("campaign is not over")]
    CampaignNotOver,
    #[msg("campaign over")]
    CampaignOver,
    #[msg("donation completed")]
    DonationCompleted,
    #[msg("donations is not completed")]
    DonationNotCompleted,
    #[msg("donations has already claimed")]
    DonationsClaimed,
    #[msg("signer is not authority")]
    SignerIsNotAuthority,
}
