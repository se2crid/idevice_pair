// Jackson Coxson

use idevice::{
    IdeviceError, IdeviceService,
    lockdown::LockdownClient,
    mobile_image_mounter::ImageMounter,
    usbmuxd::{UsbmuxdAddr, UsbmuxdDevice},
};

const BUILD_MANIFEST: &[u8] = include_bytes!("../DDI/BuildManifest.plist");
const DDI_IMAGE: &[u8] = include_bytes!("../DDI/Image.dmg");
const DDI_TRUSTCACHE: &[u8] = include_bytes!("../DDI/Image.dmg.trustcache");
pub async fn auto_mount(dev: UsbmuxdDevice) -> Result<(), IdeviceError> {
    let p = dev.to_provider(UsbmuxdAddr::default(), "idevice_pair");

    let mut mc = ImageMounter::connect(&p).await?;
    let images = mc.copy_devices().await?;
    if !images.is_empty() {
        return Ok(());
    }

    let mut lc = LockdownClient::connect(&p).await?;
    let ucid = lc
        .get_value("UniqueChipID", None)
        .await?
        .as_unsigned_integer()
        .unwrap();

    mc.mount_personalized(
        &p,
        DDI_IMAGE.to_vec(),
        DDI_TRUSTCACHE.to_vec(),
        BUILD_MANIFEST,
        None,
        ucid,
    )
    .await?;

    Ok(())
}
