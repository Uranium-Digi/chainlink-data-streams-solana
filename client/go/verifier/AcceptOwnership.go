// Code generated by https://github.com/gagliardetto/anchor-go. DO NOT EDIT.

package verifier

import (
	"errors"
	ag_binary "github.com/gagliardetto/binary"
	ag_solanago "github.com/gagliardetto/solana-go"
	ag_format "github.com/gagliardetto/solana-go/text/format"
	ag_treeout "github.com/gagliardetto/treeout"
)

// AcceptOwnership is the `acceptOwnership` instruction.
type AcceptOwnership struct {

	// [0] = [WRITE] verifierAccount
	//
	// [1] = [SIGNER] owner
	ag_solanago.AccountMetaSlice `bin:"-"`
}

// NewAcceptOwnershipInstructionBuilder creates a new `AcceptOwnership` instruction builder.
func NewAcceptOwnershipInstructionBuilder() *AcceptOwnership {
	nd := &AcceptOwnership{
		AccountMetaSlice: make(ag_solanago.AccountMetaSlice, 2),
	}
	return nd
}

// SetVerifierAccountAccount sets the "verifierAccount" account.
func (inst *AcceptOwnership) SetVerifierAccountAccount(verifierAccount ag_solanago.PublicKey) *AcceptOwnership {
	inst.AccountMetaSlice[0] = ag_solanago.Meta(verifierAccount).WRITE()
	return inst
}

// GetVerifierAccountAccount gets the "verifierAccount" account.
func (inst *AcceptOwnership) GetVerifierAccountAccount() *ag_solanago.AccountMeta {
	return inst.AccountMetaSlice.Get(0)
}

// SetOwnerAccount sets the "owner" account.
func (inst *AcceptOwnership) SetOwnerAccount(owner ag_solanago.PublicKey) *AcceptOwnership {
	inst.AccountMetaSlice[1] = ag_solanago.Meta(owner).SIGNER()
	return inst
}

// GetOwnerAccount gets the "owner" account.
func (inst *AcceptOwnership) GetOwnerAccount() *ag_solanago.AccountMeta {
	return inst.AccountMetaSlice.Get(1)
}

func (inst AcceptOwnership) Build() *Instruction {
	return &Instruction{BaseVariant: ag_binary.BaseVariant{
		Impl:   inst,
		TypeID: Instruction_AcceptOwnership,
	}}
}

// ValidateAndBuild validates the instruction parameters and accounts;
// if there is a validation error, it returns the error.
// Otherwise, it builds and returns the instruction.
func (inst AcceptOwnership) ValidateAndBuild() (*Instruction, error) {
	if err := inst.Validate(); err != nil {
		return nil, err
	}
	return inst.Build(), nil
}

func (inst *AcceptOwnership) Validate() error {
	// Check whether all (required) accounts are set:
	{
		if inst.AccountMetaSlice[0] == nil {
			return errors.New("accounts.VerifierAccount is not set")
		}
		if inst.AccountMetaSlice[1] == nil {
			return errors.New("accounts.Owner is not set")
		}
	}
	return nil
}

func (inst *AcceptOwnership) EncodeToTree(parent ag_treeout.Branches) {
	parent.Child(ag_format.Program(ProgramName, ProgramID)).
		//
		ParentFunc(func(programBranch ag_treeout.Branches) {
			programBranch.Child(ag_format.Instruction("AcceptOwnership")).
				//
				ParentFunc(func(instructionBranch ag_treeout.Branches) {

					// Parameters of the instruction:
					instructionBranch.Child("Params[len=0]").ParentFunc(func(paramsBranch ag_treeout.Branches) {})

					// Accounts of the instruction:
					instructionBranch.Child("Accounts[len=2]").ParentFunc(func(accountsBranch ag_treeout.Branches) {
						accountsBranch.Child(ag_format.Meta("verifier", inst.AccountMetaSlice.Get(0)))
						accountsBranch.Child(ag_format.Meta("   owner", inst.AccountMetaSlice.Get(1)))
					})
				})
		})
}

func (obj AcceptOwnership) MarshalWithEncoder(encoder *ag_binary.Encoder) (err error) {
	return nil
}
func (obj *AcceptOwnership) UnmarshalWithDecoder(decoder *ag_binary.Decoder) (err error) {
	return nil
}

// NewAcceptOwnershipInstruction declares a new AcceptOwnership instruction with the provided parameters and accounts.
func NewAcceptOwnershipInstruction(
	// Accounts:
	verifierAccount ag_solanago.PublicKey,
	owner ag_solanago.PublicKey) *AcceptOwnership {
	return NewAcceptOwnershipInstructionBuilder().
		SetVerifierAccountAccount(verifierAccount).
		SetOwnerAccount(owner)
}
