use contextra_storage_ports::{
    DataClass, DurablePosition, EraseRequestId, InstanceId, SpaceId, Ticket,
    TombId, FIRST_USER_SPACE_ID,
};
use contextra_storage_ports::traits::{
    Anchor, AnchorStrength, AnchorValue, Authority, FourEyesProof, Principal,
};

struct MockAnchor;

impl Anchor for MockAnchor {
    fn read(&self) -> Result<AnchorValue, contextra_storage_ports::Error> { Ok(AnchorValue::default()) }

    fn advance(
        &self,
        _: AnchorValue,
        _: AnchorValue,
    ) -> Result<(), contextra_storage_ports::Error> { Ok(()) }

    fn strength(&self) -> AnchorStrength { AnchorStrength::Strong }
}

struct MockAuthority;

impl Authority for MockAuthority {
    fn verify_four_eyes(
        &self,
        requester: &Principal,
        approver: &Principal,
        _: &FourEyesProof,
    ) -> Result<(), contextra_storage_ports::Error> {
        if requester.differs_from(approver) { Ok(()) } else { Err(contextra_storage_ports::Error::Restricted) }
    }
}

#[test]
fn r8_port_identity_types_are_opaque_and_space_ids_are_u64() {
    // INV-ID-1 INV-ID-2
    assert_eq!(SpaceId::user(FIRST_USER_SPACE_ID).ok(), Some(SpaceId(256)));
    assert!(SpaceId::user(255).is_err());

    let instance = InstanceId::from_wire([3; 16]);
    let position = DurablePosition { instance, stamp: 7 };
    assert_eq!(position.stamp, 7);
    assert_eq!(format!("{instance:?}"), "InstanceId(<redacted>)");
    assert_eq!(format!("{:?}", EraseRequestId::from_bytes([4; 16])), "EraseRequestId(<redacted>)");
    assert_eq!(format!("{:?}", Ticket::from_bytes([5; 16])), "Ticket(<redacted>)");
    assert_eq!(format!("{:?}", TombId::from_bytes([6; 32])), "TombId(<redacted>)");
    assert_eq!(DataClass::Personal, DataClass::Personal);
    assert_eq!(MockAnchor.strength(), AnchorStrength::Strong);
    let requester = Principal::from_opaque(vec![1]);
    let approver = Principal::from_opaque(vec![2]);
    assert!(MockAuthority.verify_four_eyes(&requester, &approver, &FourEyesProof::from_opaque(vec![3])).is_ok());
}
