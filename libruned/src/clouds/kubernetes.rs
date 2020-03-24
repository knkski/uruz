use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::Error;
use crate::traits::{Applier, Cloud, Translator};
use rune::Rune;

struct Kubernetes {}

impl Cloud for Kubernetes {
    fn name(&self) -> String {
        "Kubernetes".into()
    }

    fn create(&self) -> Result<(), Error> {
        Ok(())
    }
}

struct KubernetesTranslator {}

impl Translator for KubernetesTranslator {
    type Cloud = Kubernetes;
    type Output = String; // Just stringified yaml for now

    fn translate(&self, rune: &Rune) -> Result<Self::Output, Error> {
        Ok(format!(
            r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  labels:
    app: nginx
spec:
  replicas: 1
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: {}
        image: {:?}
        "#,
            rune.template[0].name, rune.template[0].image
        ))
    }
}

struct KubernetesApplier {}

impl Applier for KubernetesApplier {
    type Cloud = Kubernetes;
    type Input = String;

    fn apply(&self, input: Self::Input) -> Result<(), Error> {
        let mut child = Command::new("kubectl")
            .args(&["apply", "-f", "-"])
            .stdin(Stdio::piped())
            .spawn()
            .expect("AHHH!");

        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");

        child.wait().unwrap();

        Ok(())
    }
}

/* #[cfg(test)]
 * mod test {
 *     use super::*;
 *     use crate::rune::Metadata;
 *
 *     #[test]
 *     fn test_basic() {
 *         let cloud = Kubernetes {};
 *         let rune = Rune {
 *             metadata: Metadata {
 *                 name: "nginx".into(),
 *                 description: "nginx".into(),
 *             },
 *             containers: vec![Container {
 *                 name: "nginx".into(),
 *                 image: Image::OciImage("nginx:1.7.9".into()),
 *                 ports: vec![],
 *                 command: None,
 *                 args: None,
 *                 environment: None,
 *                 include: "".to_string(),
 *             }],
 *         };
 *         let translator = KubernetesTranslator {};
 *         let applier = KubernetesApplier {};
 *
 *         let translated = translator.translate(&rune).unwrap();
 *         let applied = applier.apply(translated).unwrap();
 *     }
 * } */
