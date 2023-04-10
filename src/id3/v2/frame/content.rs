use crate::error::{ID3v2Error, ID3v2ErrorKind, Result};
use crate::id3::v2::frame::FrameValue;
use crate::id3::v2::items::{
	AttachedPictureFrame, ExtendedTextFrame, ExtendedUrlFrame, LanguageFrame, Popularimeter,
	UniqueFileIdentifierFrame,
};
use crate::id3::v2::ID3v2Version;
use crate::macros::err;
use crate::util::text::{decode_text, TextEncoding};

use std::io::Read;

use byteorder::ReadBytesExt;

#[rustfmt::skip]
pub(super) fn parse_content(
	content: &mut &[u8],
	id: &str,
	version: ID3v2Version,
) -> Result<Option<FrameValue>> {
	Ok(match id {
		// The ID was previously upgraded, but the content remains unchanged, so version is necessary
		"APIC" => {
			let attached_picture = AttachedPictureFrame::parse(content, version)?;
			Some(FrameValue::Picture(attached_picture))
		},
		"TXXX" => ExtendedTextFrame::parse(content, version)?.map(FrameValue::UserText),
		"WXXX" => ExtendedUrlFrame::parse(content, version)?.map(FrameValue::UserURL),
		"COMM" | "USLT" => parse_text_language(content, id, version)?,
		"UFID" => UniqueFileIdentifierFrame::parse(content)?.map(FrameValue::UniqueFileIdentifier),
		_ if id.starts_with('T') => parse_text(content, version)?,
		// Apple proprietary frames
		// WFED (Podcast URL), GRP1 (Grouping), MVNM (Movement Name), MVIN (Movement Number)
		"WFED" | "GRP1" | "MVNM" | "MVIN" => parse_text(content, version)?,
		_ if id.starts_with('W') => parse_link(content)?,
		"POPM" => Some(FrameValue::Popularimeter(Popularimeter::parse(content)?)),
		// SYLT, GEOB, and any unknown frames
		_ => Some(FrameValue::Binary(content.to_vec())),
	})
}

fn parse_text_language(
	content: &mut &[u8],
	id: &str,
	version: ID3v2Version,
) -> Result<Option<FrameValue>> {
	if content.len() < 5 {
		return Ok(None);
	}

	let encoding = verify_encoding(content.read_u8()?, version)?;

	let mut language = [0; 3];
	content.read_exact(&mut language)?;

	let description = decode_text(content, encoding, true)?;
	let content = decode_text(content, encoding, false)?.unwrap_or_default();

	let information = LanguageFrame {
		encoding,
		language,
		description: description.unwrap_or_default(),
		content,
	};

	let value = match id {
		"COMM" => FrameValue::Comment(information),
		"USLT" => FrameValue::UnSyncText(information),
		_ => unreachable!(),
	};

	Ok(Some(value))
}

fn parse_text(content: &mut &[u8], version: ID3v2Version) -> Result<Option<FrameValue>> {
	if content.len() < 2 {
		return Ok(None);
	}

	let encoding = verify_encoding(content.read_u8()?, version)?;
	let text = decode_text(content, encoding, true)?.unwrap_or_default();

	Ok(Some(FrameValue::Text {
		encoding,
		value: text,
	}))
}

fn parse_link(content: &mut &[u8]) -> Result<Option<FrameValue>> {
	if content.is_empty() {
		return Ok(None);
	}

	let link = decode_text(content, TextEncoding::Latin1, true)?.unwrap_or_default();

	Ok(Some(FrameValue::URL(link)))
}

pub(in crate::id3::v2) fn verify_encoding(
	encoding: u8,
	version: ID3v2Version,
) -> Result<TextEncoding> {
	if version == ID3v2Version::V2 && (encoding != 0 && encoding != 1) {
		return Err(ID3v2Error::new(ID3v2ErrorKind::V2InvalidTextEncoding).into());
	}

	match TextEncoding::from_u8(encoding) {
		None => err!(TextDecode("Found invalid encoding")),
		Some(e) => Ok(e),
	}
}
