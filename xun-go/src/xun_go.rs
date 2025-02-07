#![no_std]
use multiversx_sc::derive_imports::*;
#[allow(unused_imports)]
use multiversx_sc::imports::*;

// Definición de las habilidades permitidas en el sistema
#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, PartialEq)]
pub enum SkillName {
    Skill1,
    Skill2,
    Skill3,
    Skill4,
    Skill5,
}

// Definición de los niveles de habilidad disponibles
#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, PartialEq)]
pub enum SkillLevel {
    Basic,
    Intermediate,
    Advanced,
}

// Estructura para representar una habilidad con su nivel correspondiente
#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, PartialEq)]
pub struct Skill {
    pub name: SkillName,
    pub level: SkillLevel,
}

// Estructura del Certificado, que asocia un usuario con una habilidad certificada
#[type_abi]
#[derive(TopEncode, TopDecode, Clone, PartialEq)]
pub struct Certificate<M: ManagedTypeApi> {
    pub id: u64, // Identificador único del certificado
    pub user: ManagedAddress<M>, // Dirección del usuario certificado
    pub issuer: ManagedAddress<M>, // Dirección del emisor del certificado
    pub skill: Skill, // Habilidad certificada
    pub issued_on: u64, // Timestamp de emisión
    pub valid: bool, // Indica si el certificado es válido o ha sido revocado
}

#[multiversx_sc::contract]
pub trait XunGo {
    #[init]
    fn init(&self) {
        self.certificate_counter().set(0u64); // Inicializa el contador de certificados
        log!("Contrato inicializado");
    }

    

    // Método para certificar una habilidad a un usuario
    #[endpoint(certifySkill)]
    fn certify_skill(&self, user: ManagedAddress, skill_name: SkillName, level: SkillLevel) {
        let caller = self.blockchain().get_caller(); // Obtiene la dirección del emisor
        let new_id = self.certificate_counter().get() + 1; // Genera un nuevo ID único
        log!("Certificando habilidad: {:?} para usuario {:?} con nivel {:?}", skill_name, user, level);
        let skill = Skill {
            name: skill_name,
            level,
        };

        let cert = Certificate {
            id: new_id,
            user: user.clone(),
            issuer: caller,
            skill,
            issued_on: self.blockchain().get_block_timestamp(), // Asigna la fecha de emisión
            valid: true,
        };

        self.certificates(new_id).set(&cert); // Guarda el certificado
        self.user_certificates(&user).push(&new_id); // Asocia el certificado con el usuario
        self.certificate_counter().set(new_id); // Actualiza el contador de certificados
        log!("Certificado emitido con ID: {:?}", new_id);
    }

    // Método para obtener los certificados válidos de un usuario
    #[view(getUserSkills)]
   fn get_user_skills(&self, user: ManagedAddress) -> MultiValueEncoded<Certificate<Self::Api>> {
       let mut certificates = MultiValueEncoded::new();
       log!("Obteniendo certificados para usuario: {:?}", user);
          for cert_id in self.user_certificates(&user).iter() {
            if !self.certificates(cert_id).is_empty() {
                let cert = self.certificates(cert_id).get();
                log!("Encontrado certificado ID: {:?}, válido: {:?}", cert.id, cert.valid);
                if cert.valid {
                    certificates.push(cert); // Solo se devuelven certificados válidos
                }
            }
        }
        log!("Total certificados válidos encontrados: {:?}", certificates.len());
        certificates
    }

    // Método para revocar un certificado por parte del emisor original
    #[endpoint(revokeCertificate)]
    fn revoke_certificate(&self, cert_id: u64) {
        let caller = self.blockchain().get_caller();
        require!(!self.certificates(cert_id).is_empty(), "Certificado no encontrado");
        
        let mut cert = self.certificates(cert_id).get();
        require!(cert.issuer == caller, "Solo el emisor puede revocar el certificado"); // Verifica que el emisor sea el mismo que emitió el certificado
        cert.valid = false; // Marca el certificado como inválido
        self.certificates(cert_id).set(&cert); // Guarda el cambio
        log!("Certificado ID {:?} revocado por {:?}", cert_id, caller);
    }


// Mappers para almacenamiento de datos
#[view(getCertificates)]
#[storage_mapper("certificates")]
fn certificates(&self, id: u64) -> SingleValueMapper<Certificate<Self::Api>>;

#[view(userCertificates)]
#[storage_mapper("user_certificates")]
fn user_certificates(&self, user: &ManagedAddress) -> VecMapper<u64>;

#[view(certificateCounter)]
#[storage_mapper("certificate_counter")]
fn certificate_counter(&self) -> SingleValueMapper<u64>;






}
