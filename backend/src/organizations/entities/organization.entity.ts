import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';
import { OrganizationVerificationStatus } from '../enums/organization-verification-status.enum';

@Entity('organizations')
export class OrganizationEntity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 200 })
  name: string;

  @Column({ name: 'legal_name', type: 'varchar', length: 200 })
  legalName: string;

  @Column({ type: 'varchar', length: 255 })
  email: string;

  @Column({ type: 'varchar', length: 32 })
  phone: string;

  @Column({ type: 'text', nullable: true })
  address: string | null;

  @Column({ name: 'license_number', type: 'varchar', length: 100, unique: true })
  licenseNumber: string;

  @Column({
    type: 'varchar',
    length: 40,
    default: OrganizationVerificationStatus.PENDING_VERIFICATION,
  })
  status: OrganizationVerificationStatus;

  @Column({ name: 'license_document_path', type: 'varchar', length: 512 })
  licenseDocumentPath: string;

  @Column({ name: 'certificate_document_path', type: 'varchar', length: 512 })
  certificateDocumentPath: string;

  @Column({ name: 'rejection_reason', type: 'text', nullable: true })
  rejectionReason: string | null;

  @Column({ name: 'verified_at', type: 'timestamp', nullable: true })
  verifiedAt: Date | null;

  @Column({ name: 'verified_by_user_id', type: 'uuid', nullable: true })
  verifiedByUserId: string | null;

  /** Soroban / Stellar transaction hash recorded when the org is anchored on-chain. */
  @Column({ name: 'blockchain_tx_hash', type: 'varchar', length: 128, nullable: true })
  blockchainTxHash: string | null;

  /** Registry or contract identifier used for verified orgs (e.g. Soroban contract id). */
  @Column({ name: 'blockchain_address', type: 'varchar', length: 128, nullable: true })
  blockchainAddress: string | null;

  @CreateDateColumn({ name: 'created_at' })
  createdAt: Date;

  @UpdateDateColumn({ name: 'updated_at' })
  updatedAt: Date;
}
