import { AnimationManager } from './animation-manager';
import { GameController, BotPlayer } from './controller';

export interface RuntimeModule {
  init(): boolean;
}

class Runtime {
  public animationManager: AnimationManager;
  public gameController: GameController | null = null;

  private modules: RuntimeModule[];

  public constructor() {
    this.animationManager = new AnimationManager();
    this.modules = [
      this.animationManager,
    ];
  }

  public addModule(module: RuntimeModule): void {
    this.modules.push(module);
  }

  public init(): boolean {
    for (const module of this.modules) {
      if (!module.init()) {
        return false;
      }
    }

    return true;
  }

  // public tick(): void {
  //   for (const module of this.modules) {
  //     module.tick();
  //   }
  // }
}

export const runtime = new Runtime();
