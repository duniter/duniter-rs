//  Copyright (C) 2018  The Durs Project Developers.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Define the Text Document Traits.

use crate::*;
use dup_crypto::keys::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Contains a document in full or compact format
pub enum TextDocumentFormat<D: TextDocument> {
    /// Complete format (Allows to check the validity of the signature)
    Complete(D),
    /// Format present in the blocks (does not always allow to verify the signature)
    Compact(D::CompactTextDocument_),
}

impl<D: TextDocument> TextDocumentFormat<D> {
    /// To compact document
    pub fn to_compact_document(&self) -> D::CompactTextDocument_ {
        match *self {
            TextDocumentFormat::Complete(ref doc) => doc.to_compact_document(),
            TextDocumentFormat::Compact(ref compact_doc) => (*compact_doc).clone(),
        }
    }
}

/// Trait for a compact text document.
pub trait CompactTextDocument: Sized + Clone {
    /// Generate document compact text.
    /// the compact format is the one used in the blocks.
    ///
    /// - Don't contains leading signatures
    /// - Contains line breaks on all line.
    fn as_compact_text(&self) -> String;
}

impl<D: TextDocument> CompactTextDocument for TextDocumentFormat<D> {
    fn as_compact_text(&self) -> String {
        match *self {
            TextDocumentFormat::Complete(ref doc) => doc.generate_compact_text(),
            TextDocumentFormat::Compact(ref doc) => doc.as_compact_text(),
        }
    }
}

/// Trait for a V10 document.
pub trait TextDocument: Document<PublicKey = PubKey, CurrencyType = str> {
    /// Type of associated compact document.
    type CompactTextDocument_: CompactTextDocument;

    /// Return document as text.
    fn as_text(&self) -> &str;

    /// Return document as text without signature.
    fn as_text_without_signature(&self) -> &str {
        let text = self.as_text();
        let mut lines: Vec<&str> = self.as_text().split('\n').collect();
        let sigs = self.signatures();
        let mut sigs_str_len = sigs.len() - 1;
        for _ in sigs {
            sigs_str_len += lines.pop().unwrap_or("").len();
        }
        &text[0..(text.len() - sigs_str_len)]
    }

    /*/// Return document as text with leading signatures.
    fn as_text_with_signatures(&self) -> String {
        let mut text = self.as_text().to_string();

        for sig in self.signatures() {
            text = format!("{}{}\n", text, sig.to_base64());
        }

        text
    }*/

    /// Generate compact document.
    /// the compact format is the one used in the blocks.
    /// - Don't contains leading signatures
    fn to_compact_document(&self) -> Self::CompactTextDocument_;

    /// Generate document compact text.
    /// the compact format is the one used in the blocks.
    ///
    /// - Don't contains leading signatures
    /// - Contains line breaks on all line.
    fn generate_compact_text(&self) -> String {
        self.to_compact_document().as_compact_text()
    }
}

/// Trait for a V10 document builder.
pub trait TextDocumentBuilder: DocumentBuilder {
    /// Generate document text.
    ///
    /// - Don't contains leading signatures
    /// - Contains line breaks on all line.
    fn generate_text(&self) -> String;

    /// Generate final document with signatures, and also return them in an array.
    ///
    /// Returns :
    ///
    /// - Text without signatures
    /// - Signatures
    fn build_signed_text(&self, private_keys: Vec<PrivKey>) -> (String, Vec<Sig>) {
        let text = self.generate_text();

        let signatures: Vec<_> = {
            let text_bytes = text.as_bytes();
            private_keys
                .iter()
                .map(|key| key.sign(text_bytes))
                .collect()
        };

        (text, signatures)
    }
}
