import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';

import { BlockchainModule } from '../blockchain/blockchain.module';
import { CompensationModule } from '../common/compensation/compensation.module';
import { InventoryModule } from '../inventory/inventory.module';
import { NotificationsModule } from '../notifications/notifications.module';

import { BloodRequestsController } from './blood-requests.controller';
import { BloodRequestsService } from './blood-requests.service';
import { RequestQueryController } from './controllers/request-query.controller';
import { OrderSplittingController } from './controllers/order-splitting.controller';
import { BloodRequestItemEntity } from './entities/blood-request-item.entity';
import { BloodRequestEntity } from './entities/blood-request.entity';
import { FulfillmentLegEntity } from './entities/fulfillment-leg.entity';
import { RequestQueryService } from './services/request-query.service';
import { OrderSplittingService } from './services/order-splitting.service';

@Module({
  imports: [
    TypeOrmModule.forFeature([
      BloodRequestEntity,
      BloodRequestItemEntity,
      FulfillmentLegEntity,
    ]),
    InventoryModule,
    BlockchainModule,
    NotificationsModule,
    CompensationModule,
  ],
  controllers: [
    BloodRequestsController,
    RequestQueryController,
    OrderSplittingController,
  ],
  providers: [BloodRequestsService, RequestQueryService, OrderSplittingService],
  exports: [BloodRequestsService, RequestQueryService, OrderSplittingService],
})
export class BloodRequestsModule {}
