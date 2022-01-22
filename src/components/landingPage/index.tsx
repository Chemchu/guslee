import { motion } from "framer-motion";
import { ScreenSide } from "../../../types";
import LateralCard from "../card";

const LandingPage = () => {

    const smallScreenImages = {
        container: `flex flex-col`,
        images: `h-72 w-72`
    }

    const mdScreenImages = {
        container: `md:grid md:grid-cols-2 md:justify-items-stretch md:gap-12 md:bg-cyan-500`,
        images: `md:w-92 md:w-92`
    }

    return (
        <motion.main className="dark:bg-gray-800 font-outfit-light bg-white relative overflow-y-scroll overflow-x-hidden h-screen"
            initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
            {/* <header className="h-24 sm:h-32 flex items-center z-30 w-full">
                <div className="container mx-auto px-6 flex items-center justify-between">
                    <div className="uppercase text-gray-800 dark:text-white font-black text-3xl flex items-center">
                        <svg width="25" height="25" viewBox="0 0 1792 1792" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
                            <path d="M1664 1504v-768q-32 36-69 66-268 206-426 338-51 43-83 67t-86.5 48.5-102.5 24.5h-2q-48 0-102.5-24.5t-86.5-48.5-83-67q-158-132-426-338-37-30-69-66v768q0 13 9.5 22.5t22.5 9.5h1472q13 0 22.5-9.5t9.5-22.5zm0-1051v-24.5l-.5-13-3-12.5-5.5-9-9-7.5-14-2.5h-1472q-13 0-22.5 9.5t-9.5 22.5q0 168 147 284 193 152 401 317 6 5 35 29.5t46 37.5 44.5 31.5 50.5 27.5 43 9h2q20 0 43-9t50.5-27.5 44.5-31.5 46-37.5 35-29.5q208-165 401-317 54-43 100.5-115.5t46.5-131.5zm128-37v1088q0 66-47 113t-113 47h-1472q-66 0-113-47t-47-113v-1088q0-66 47-113t113-47h1472q66 0 113 47t47 113z">
                            </path>
                        </svg>
                        <span className="text-xs ml-3 mt-1">
                            gustavolee26@gmail.com
                        </span>
                    </div>
                    <div className="flex items-center">
                        <nav className="font-sen text-gray-800 dark:text-white uppercase text-lg lg:flex items-center hidden">
                            <a href="#" className="py-2 px-6 flex hover:text-black">
                                Proyectos
                            </a>
                            <a href="#" className="py-2 px-6 flex hover:text-black">
                                Curriculum
                            </a>
                            <a href="#" className="py-2 px-6 flex hover:text-black">
                                Twitter
                            </a>
                            <a href="#" className="py-2 px-6 flex hover:text-black">
                                Instagram
                            </a>
                        </nav>
                        <button className="lg:hidden flex flex-col ml-4">
                            <span className="w-6 h-1 bg-gray-800 dark:bg-white mb-1">
                            </span>
                            <span className="w-6 h-1 bg-gray-800 dark:bg-white mb-1">
                            </span>
                            <span className="w-6 h-1 bg-gray-800 dark:bg-white mb-1">
                            </span>
                        </button>
                    </div>
                </div>
            </header> */}
            <div className="flex relative z-20 items-center">
                <div className="container mx-auto px-6 flex flex-col justify-between items-center relative py-4">
                    <div className="flex flex-col">
                        <motion.p initial={{ opacity: 0, x: '-30vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 0 } }}
                            className="text-3xl my-6 text-center dark:text-white">
                            Buenas, me llamo Gustavo
                        </motion.p>
                        <motion.p initial={{ opacity: 0, x: '30vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1.5 } }}
                            className="text-3xl my-6 text-center dark:text-white">
                            Me gusta programar
                        </motion.p>
                        <motion.h2 whileHover={{ scale: 1.1 }} className="max-w-3xl text-5xl md:text-6xl font-bold mx-auto dark:text-white text-gray-800 text-center py-2">
                            Portfolio a medio hacer...
                        </motion.h2>
                    </div>
                </div>
            </div>
            <div className={`${smallScreenImages.container} ${mdScreenImages.container} h-full w-full`}>
                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-start`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus1.jpeg" side={ScreenSide.LeftSide} />
                </div>
                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-end`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus2.jpg" side={ScreenSide.RightSide} />
                </div>

                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-start`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus1.jpeg" side={ScreenSide.LeftSide} />
                </div>
                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-end`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus2.jpg" side={ScreenSide.RightSide} />
                </div>
                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-start`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus1.jpeg" side={ScreenSide.LeftSide} />
                </div>
                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-end`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus2.jpg" side={ScreenSide.RightSide} />
                </div>
                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-start`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus1.jpeg" side={ScreenSide.LeftSide} />
                </div>
                <div className={`${mdScreenImages.images} ${smallScreenImages.images} md:justify-self-end`}>
                    <LateralCard className="rounded-3xl" imgSrc="/img/gus2.jpg" side={ScreenSide.RightSide} />
                </div>


            </div>
        </motion.main>

    );
}

export default LandingPage;